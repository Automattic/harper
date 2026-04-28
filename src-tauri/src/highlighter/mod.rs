mod error;
mod window;
mod window_manager;

pub use error::Error;
use window_manager::WindowManager;

pub struct Highlighter {
    window_manager: WindowManager,
}

impl Highlighter {
    pub fn run_window_for_each_monitor() -> Result<(), Error> {
        Self {
            window_manager: WindowManager::new()?,
        }
        .window_manager
        .run_window_for_each_monitor()
    }
}
