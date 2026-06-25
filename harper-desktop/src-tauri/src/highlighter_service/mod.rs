mod highlighter_process;
mod highlighter_worker;

use crate::config::Config;
use highlighter_worker::HighlighterWorker;
use std::{
    io,
    sync::{Arc, Mutex as StdMutex},
};
use tokio::sync::Mutex;

/// Wraps around a [`HighlighterWorker`] and turns it into a service that is easier to manage.
/// We can start and stop it and simply provide a shared `Config` object that we can update whenever.
/// The service will sync the changes to the underlying highlighter process.
pub struct HighlighterService {
    config: Arc<Mutex<Config>>,
    worker: StdMutex<Option<HighlighterWorker>>,
}

impl HighlighterService {
    /// Create a new highlighter process service, without starting it.
    /// The provided config pointer will be automatically synced to the new process when changes are
    /// made.
    pub fn new(config: Arc<Mutex<Config>>) -> Self {
        Self {
            config,
            worker: StdMutex::new(None),
        }
    }

    /// Starts the highlighter worker if it is not already running.
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
    pub fn is_running(&self) -> bool {
        self.reap_finished_worker();

        self.worker
            .lock()
            .expect("highlighter service lock poisoned")
            .is_some()
    }

    /// Joins and removes a worker whose thread has already exited.
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
