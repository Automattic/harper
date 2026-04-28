mod error;
mod window;
mod window_manager;

pub use error::Error;
use window_manager::WindowManager;

pub struct Highlighter {
    context: egui::Context,
    window_manager: WindowManager,
}

impl Highlighter {
    pub fn new() -> Result<Self, Error> {
        let context = egui::Context::default();

        Ok(Self {
            window_manager: WindowManager::new(context.clone())?,
            context,
        })
    }

    pub fn run_window_for_each_monitor(self) -> Result<(), Error> {
        let Self {
            context,
            window_manager,
        } = self;

        drop(context);

        window_manager.run_window_for_each_monitor()
    }
}
