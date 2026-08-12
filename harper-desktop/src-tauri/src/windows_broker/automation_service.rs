use std::thread::{JoinHandle, sleep};
use std::sync::mpsc::{Receiver, TrySendError, sync_channel};
use std::time::{Duration, Instant};

use uiautomation::{UIAutomation, UIElement, patterns::UITextPattern};

struct WorkerData {
    thread_handle: JoinHandle<()>,
    receiver: Receiver<(String, Instant)>
}

/// Runs and communicates with a worker thread to interact with the Win32 Automation API to query the accessibility tree.
/// Necessary because the API has very specific thread setting requirements to work. 
pub struct AutomationService{
    worker_data: Option<WorkerData>
}

impl AutomationService {
    pub fn create_and_start() -> Self{
        let mut output = Self {
            worker_data: None
        };

        output.start_worker_thread();

        output
    }

    /// Starts the worker thread if it is not already running.
    /// Does nothing if the worker thread is already running.
    fn start_worker_thread(&mut self){
        let (sender, receiver) = sync_channel(1);

        let handle = std::thread::spawn(move ||{
            let automation = UIAutomation::new().unwrap();
            loop{
                let root = automation.get_focused_element().unwrap();

                if let Ok(text) = get_text(&root){
                    // Stop the thread if the other side of the channel has been closed (or dropped).
                    if let Err(TrySendError::Disconnected(_)) = sender.try_send((text, Instant::now())){
                        break;
                    }
                }

                sleep(Duration::from_millis(16));
            }
        });

        self.worker_data = Some(WorkerData {receiver, thread_handle: handle});
    }

    /// Stops the worker thread if it is running. This method does nothing if it is not running.
    fn stop_worker_thread(&mut self) {
        // This drops the inner fields, which closes the channel, which signals to the worker to stop running.
       self.worker_data = None;
    }

    /// Grab text from the worker.
    /// Attempts to get the most up-to-date information possible.
    /// Returns `None` if the worker is not running.
    pub fn get_text(&self) -> Option<String>{
       let worker_data = self.worker_data.as_ref()?;
       let (mesg, time) = worker_data.receiver.recv().unwrap();
       
       if (Instant::now().duration_since(time) > Duration::from_millis(16)){
        let (mesg, time) = worker_data.receiver.recv().unwrap();
        return Some(mesg);
       }
       return Some(mesg);
    }
}

fn get_text(element: &UIElement) -> uiautomation::Result<String> {
    let pattern: UITextPattern = element.get_pattern()?;
    let range = pattern.get_document_range()?;
    range.get_text(-1)
}