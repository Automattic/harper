mod error;
mod render_state;
mod window;
mod window_manager;

use std::time::Duration;

pub use error::Error;
use window_manager::WindowManager;

use crate::rect::ColoredRect;

const DEFAULT_READ_INTERVAL: Duration = Duration::from_millis(100);

pub type ReadRects = Box<dyn FnMut() -> Option<Vec<ColoredRect>>>;

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
    pub fn new(
        read_rects: impl FnMut() -> Option<Vec<ColoredRect>> + 'static,
    ) -> Result<Self, Error> {
        let context = egui::Context::default();

        Ok(Self {
            window_manager: WindowManager::new(
                context.clone(),
                Box::new(read_rects),
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

    pub fn set_rects(&mut self, rects: Vec<ColoredRect>) {
        self.window_manager.set_rects(rects);
    }
}
