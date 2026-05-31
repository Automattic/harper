use crate::communication::Server;
use crate::config::Config;
use crate::highlighter_process::HighlighterProcess;
use std::{
    io,
    sync::{Arc, Mutex as StdMutex},
    thread::{self, JoinHandle},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    runtime::Builder,
    sync::{Mutex, oneshot},
};

/// Owns the optional highlighter worker for the Tauri app.
///
/// This service gives app and tray code a small start/stop/toggle API without exposing the worker
/// thread, child process, or IPC server details.
pub struct HighlighterService {
    config: Arc<Mutex<Config>>,
    worker: StdMutex<Option<HighlighterWorker>>,
}

impl HighlighterService {
    /// Creates a service controller around shared app config.
    ///
    /// The service keeps the config so each newly started highlighter process serves IPC from the
    /// same state used by Tauri commands.
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        Self {
            config,
            worker: StdMutex::new(None),
        }
    }

    /// Starts the highlighter worker if it is not already running.
    ///
    /// This is idempotent so callers can request startup from app boot or tray actions without
    /// risking duplicate child processes. Returns whether the service is running.
    pub fn start(&self) -> io::Result<bool> {
        self.reap_finished_worker();

        let mut worker = self
            .worker
            .lock()
            .expect("highlighter service lock poisoned");
        if worker.is_some() {
            return Ok(true);
        }

        *worker = Some(HighlighterWorker::spawn(self.config.clone())?);

        Ok(true)
    }

    /// Stops the highlighter worker if one is running.
    ///
    /// This takes ownership of the current worker before shutting it down so future status checks see
    /// the service as stopped immediately. Returns whether the service is running.
    pub fn stop(&self) -> bool {
        let worker = self
            .worker
            .lock()
            .expect("highlighter service lock poisoned")
            .take();

        if let Some(mut worker) = worker {
            worker.stop();
        }

        false
    }

    /// Starts or stops the worker based on the current state.
    ///
    /// The tray menu uses this as its single action for the highlighter service. Returns whether the
    /// service is running after the toggle completes.
    pub fn toggle(&self) -> io::Result<bool> {
        if self.is_running() {
            Ok(self.stop())
        } else {
            self.start()
        }
    }

    /// Reports whether a live worker is currently owned by the service.
    ///
    /// This first reaps any worker whose thread has already exited so stale finished workers do not
    /// appear as active. Returns whether the service is running.
    pub fn is_running(&self) -> bool {
        self.reap_finished_worker();

        self.worker
            .lock()
            .expect("highlighter service lock poisoned")
            .is_some()
    }

    /// Joins and removes a worker whose thread has already exited.
    ///
    /// The worker can finish without an explicit stop if the child highlighter exits or closes its
    /// IPC pipe. Reaping keeps the service state accurate and prevents leaking join handles.
    fn reap_finished_worker(&self) {
        let worker = {
            let mut worker = self
                .worker
                .lock()
                .expect("highlighter service lock poisoned");
            if worker.as_ref().is_some_and(HighlighterWorker::is_finished) {
                worker.take()
            } else {
                None
            }
        };

        if let Some(mut worker) = worker {
            worker.stop();
        }
    }
}

impl Drop for HighlighterService {
    /// Stops the worker when the Tauri-managed service is dropped.
    ///
    /// This ties child-process cleanup to service ownership during application shutdown.
    fn drop(&mut self) {
        let worker = self
            .worker
            .get_mut()
            .expect("highlighter service lock poisoned");

        if let Some(mut worker) = worker.take() {
            worker.stop();
        }
    }
}

/// Runtime resources for one running highlighter service instance.
///
/// A worker owns the shutdown signal and OS thread that hosts the Tokio runtime, child highlighter
/// process, and Tauri-side IPC server.
struct HighlighterWorker {
    shutdown_sender: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl HighlighterWorker {
    /// Spawns the service thread, child highlighter process, and IPC server.
    ///
    /// The startup channel makes this method synchronous from the caller's perspective: it only
    /// returns success after the child process and server pipes are ready.
    fn spawn(config: Arc<Mutex<Config>>) -> io::Result<Self> {
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        let (startup_sender, startup_receiver) = std::sync::mpsc::sync_channel(1);

        let thread = thread::Builder::new()
            .name("harper-highlighter-service".to_string())
            .spawn(move || {
                let runtime = match Builder::new_current_thread().enable_all().build() {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        let _ = startup_sender.send(Err(io::Error::other(error)));
                        return;
                    }
                };

                runtime.block_on(async move {
                    let mut highlighter_process = match HighlighterProcess::spawn() {
                        Ok(process) => process,
                        Err(error) => {
                            let _ = startup_sender.send(Err(error));
                            return;
                        }
                    };

                    let mut server = match highlighter_process.create_server(config) {
                        Ok(server) => server,
                        Err(error) => {
                            let _ = startup_sender.send(Err(error));
                            return;
                        }
                    };

                    let _ = startup_sender.send(Ok(()));
                    run_server_until_shutdown(&mut server, shutdown_receiver).await;
                    highlighter_process.terminate().await;
                });
            })?;

        match startup_receiver.recv() {
            Ok(Ok(())) => Ok(Self {
                shutdown_sender: Some(shutdown_sender),
                thread: Some(thread),
            }),
            Ok(Err(error)) => {
                let _ = thread.join();
                Err(error)
            }
            Err(_) => {
                let _ = thread.join();
                Err(io::Error::other(
                    "highlighter service exited before reporting startup status",
                ))
            }
        }
    }

    /// Reports whether the worker thread has exited.
    ///
    /// The service uses this to lazily clean up workers that ended because the child process exited
    /// or IPC reached EOF.
    fn is_finished(&self) -> bool {
        self.thread.as_ref().is_some_and(JoinHandle::is_finished)
    }

    /// Requests shutdown and joins the worker thread.
    ///
    /// Sending the shutdown signal lets the async server loop break before the worker terminates and
    /// reaps the child process.
    fn stop(&mut self) {
        if let Some(shutdown_sender) = self.shutdown_sender.take() {
            let _ = shutdown_sender.send(());
        }

        if let Some(thread) = self.thread.take()
            && let Err(error) = thread.join()
        {
            eprintln!("highlighter service thread panicked: {error:?}");
        }
    }
}

impl Drop for HighlighterWorker {
    /// Ensures a worker cannot be dropped while its thread and child process are still running.
    fn drop(&mut self) {
        self.stop();
    }
}

/// Serves highlighter IPC requests until shutdown or child EOF.
///
/// This exists so the worker thread can race normal protocol handling against the service shutdown
/// signal without making the highlighter process aware of tray-service state.
async fn run_server_until_shutdown<R, W>(
    server: &mut Server<R, W>,
    mut shutdown_receiver: oneshot::Receiver<()>,
) where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    loop {
        tokio::select! {
            _ = &mut shutdown_receiver => break,
            request = server.receive_request() => {
                match request {
                    Ok(Some(_)) => {}
                    Ok(None) => break,
                    Err(error) => eprintln!("failed to receive highlighter request: {error}"),
                }
            }
        }
    }
}
