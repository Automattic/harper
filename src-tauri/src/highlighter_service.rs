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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlighterServiceStatus {
    Running,
    Stopped,
}

pub struct HighlighterService {
    config: Arc<Mutex<Config>>,
    worker: StdMutex<Option<HighlighterWorker>>,
}

impl HighlighterService {
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        Self {
            config,
            worker: StdMutex::new(None),
        }
    }

    pub fn start(&self) -> io::Result<HighlighterServiceStatus> {
        self.reap_finished_worker();

        let mut worker = self
            .worker
            .lock()
            .expect("highlighter service lock poisoned");
        if worker.is_some() {
            return Ok(HighlighterServiceStatus::Running);
        }

        *worker = Some(HighlighterWorker::spawn(self.config.clone())?);

        Ok(HighlighterServiceStatus::Running)
    }

    pub fn stop(&self) -> HighlighterServiceStatus {
        let worker = self
            .worker
            .lock()
            .expect("highlighter service lock poisoned")
            .take();

        if let Some(mut worker) = worker {
            worker.stop();
        }

        HighlighterServiceStatus::Stopped
    }

    pub fn toggle(&self) -> io::Result<HighlighterServiceStatus> {
        match self.status() {
            HighlighterServiceStatus::Running => Ok(self.stop()),
            HighlighterServiceStatus::Stopped => self.start(),
        }
    }

    pub fn status(&self) -> HighlighterServiceStatus {
        self.reap_finished_worker();

        if self
            .worker
            .lock()
            .expect("highlighter service lock poisoned")
            .is_some()
        {
            HighlighterServiceStatus::Running
        } else {
            HighlighterServiceStatus::Stopped
        }
    }

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

struct HighlighterWorker {
    shutdown_sender: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl HighlighterWorker {
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

    fn is_finished(&self) -> bool {
        self.thread.as_ref().is_some_and(JoinHandle::is_finished)
    }

    fn stop(&mut self) {
        if let Some(shutdown_sender) = self.shutdown_sender.take() {
            let _ = shutdown_sender.send(());
        }

        if let Some(thread) = self.thread.take() {
            if let Err(error) = thread.join() {
                eprintln!("highlighter service thread panicked: {error:?}");
            }
        }
    }
}

impl Drop for HighlighterWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

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
