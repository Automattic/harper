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

/// Represents a thread that manages the communication and state for a highlighter process.
pub struct HighlighterWorker {
    shutdown_sender: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl HighlighterWorker {
    /// Spawns a new highlighter process and thread to manage it.
    pub fn spawn(config: Arc<Mutex<Config>>) -> io::Result<Self> {
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        let (startup_sender, startup_receiver) = std::sync::mpsc::sync_channel(1);

        // Start a thread to manage communication with the new process.
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
    pub fn is_finished(&self) -> bool {
        self.thread.as_ref().is_some_and(JoinHandle::is_finished)
    }

    /// Requests shutdown and joins the worker thread.
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
/// This is the content of the thread the above struct represents.
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
