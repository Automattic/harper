mod error;
mod render_state;
mod window;
mod window_manager;

use std::time::Duration;

pub use error::Error;
use window_manager::WindowManager;

use crate::os_broker::OsBroker;
use crate::rect::PositionedLint;

const DEFAULT_READ_INTERVAL: Duration = Duration::from_millis(100);

/// Public entry point for the screen highlighter system.
///
/// `Highlighter` owns the shared egui context and delegates native window/event-loop work to the
/// window manager. Keeping this as the top-level type gives future callers one place to configure
/// or update highlighter state without depending on the windowing implementation.
pub struct Highlighter {
    context: egui::Context,
    window_manager: WindowManager,
}

impl Highlighter {
    pub fn new(os_broker: impl OsBroker + 'static) -> Result<Self, Error> {
        let context = egui::Context::default();

        Ok(Self {
            window_manager: WindowManager::new(
                context.clone(),
                Box::new(os_broker),
                DEFAULT_READ_INTERVAL,
            )?,
            context,
        })
    }

    pub fn with_read_interval(mut self, read_interval: Duration) -> Self {
        self.window_manager.set_read_interval(read_interval);
        self
    }

    pub fn run_window_for_each_monitor(self) -> Result<(), Error> {
        let Self {
            context,
            window_manager,
        } = self;

        drop(context);

        window_manager.run_window_for_each_monitor()
    }

    pub fn set_rects(&mut self, rects: Vec<PositionedLint>) {
        self.window_manager.set_rects(rects);
    }
}
