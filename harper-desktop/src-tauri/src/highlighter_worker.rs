use crate::highlighter_process::HighlighterProcessManager;
use crate::{communication::Server, config::Config};
use std::{
    io,
    sync::Arc,
    thread::{self, JoinHandle},
};
use tokio::io::AsyncWrite;
use tokio::{
    io::AsyncRead,
    runtime::Builder,
    sync::{Mutex, oneshot},
};

/// Runtime resources for one running highlighter service instance.
///
/// A worker owns the shutdown signal and OS thread that hosts the Tokio runtime, child highlighter
/// process, and Tauri-side IPC server.
pub struct HighlighterWorker {
    shutdown_sender: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl HighlighterWorker {
    /// Spawns the service thread, child highlighter process, and IPC server.
    ///
    /// The startup channel makes this method synchronous from the caller's perspective: it only
    /// returns success after the child process and server pipes are ready.
    pub fn spawn(config: Arc<Mutex<Config>>) -> io::Result<Self> {
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
                    let mut highlighter_process = match HighlighterProcessManager::spawn() {
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
    pub fn is_finished(&self) -> bool {
        self.thread.as_ref().is_some_and(JoinHandle::is_finished)
    }

    /// Requests shutdown and joins the worker thread.
    ///
    /// Sending the shutdown signal lets the async server loop break before the worker terminates and
    /// reaps the child process.
    pub fn stop(&mut self) {
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
