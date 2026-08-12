use std::sync::mpsc::{Receiver, Sender, SyncSender, TryRecvError, TrySendError, sync_channel};
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, Instant};

use is_macro::Is;
use uiautomation::{UIAutomation, UIElement, patterns::UITextPattern};

/// Information about a worker thread.
struct WorkerData {
    thread_handle: JoinHandle<()>,
    sender: SyncSender<WorkerJob>,
    receiver: Receiver<JobResult>,
}

/// The result of a job run by the worker thread.
#[derive(Debug, Is)]
enum JobResult {
    String(String),
    Err,
}

/// An actual function pointer to be run by the worker thread.
type WorkerJob = fn(&UIAutomation) -> JobResult;

/// Runs and communicates with a worker thread to interact with the Win32 Automation API to query the accessibility tree.
/// Necessary because the API has very specific thread setting requirements to work.
pub struct AutomationService {
    worker_data: Option<WorkerData>,
}

impl AutomationService {
    pub fn create_and_start() -> Self {
        let mut output = Self { worker_data: None };

        output.start_worker_thread();

        output
    }

    /// Starts the worker thread if it is not already running.
    /// Does nothing if the worker thread is already running.
    fn start_worker_thread(&mut self) {
        let (job_sender, job_receiver) = sync_channel::<WorkerJob>(4);
        let (result_sender, result_receiver) = sync_channel(1);

        let handle = std::thread::spawn(move || {
            let automation = UIAutomation::new().unwrap();
            loop {
                // Stop the thread if the other side of the channel has been closed (or dropped).
                let job = match job_receiver.try_recv() {
                    Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => None,
                    Ok(job) => Some(job),
                };

                if let Some(job) = job {
                    let result = job(&automation);

                    // Stop the thread if the other side of the channel has been closed (or dropped).
                    if let Err(TrySendError::Disconnected(_)) = result_sender.try_send(result) {
                        break;
                    }
                }

                sleep(Duration::from_millis(16));
            }
        });

        self.worker_data = Some(WorkerData {
            receiver: result_receiver,
            sender: job_sender,
            thread_handle: handle,
        });
    }

    /// Stops the worker thread if it is running. This method does nothing if it is not running.
    fn stop_worker_thread(&mut self) {
        // This drops the inner fields, which closes the channel, which signals to the worker to stop running.
        self.worker_data = None;
    }

    /// Attempts to run a worker job on the worker thread. Returns `None` if the worker thread does not exist.
    fn run_worker_job(&self, job: WorkerJob) -> Option<JobResult> {
        let worker_data = self.worker_data.as_ref()?;
        worker_data.sender.send(job).unwrap();
        Some(worker_data.receiver.recv().unwrap())
    }

    /// Grab text from the worker.
    /// Attempts to get the most up-to-date information possible.
    /// Returns `None` if the worker is not running.
    pub fn get_text(&self) -> Option<String> {
        let result = self.run_worker_job(get_text_job)?;
        result.as_string().cloned()
    }
}

fn get_text(element: &UIElement) -> uiautomation::Result<String> {
    let pattern: UITextPattern = element.get_pattern()?;
    let range = pattern.get_document_range()?;
    range.get_text(-1)
}

fn get_text_job(automation: &UIAutomation) -> JobResult {
    let root = automation.get_focused_element().unwrap();

    if let Ok(text) = get_text(&root) {
        JobResult::String(text)
    } else {
        JobResult::Err
    }
}
