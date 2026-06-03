use crate::config::Config;
use crate::highlighter_process::HighlighterProcess;
use crate::{communication::Server, highlighter_worker::HighlighterWorker};
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
