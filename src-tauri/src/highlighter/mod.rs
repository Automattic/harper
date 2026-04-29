mod error;
mod render_state;
mod window;
mod window_manager;

pub use error::Error;
use window_manager::WindowManager;

use crate::rect::Rect;

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

    pub fn set_rects(&mut self, rects: Vec<Rect>) {
        self.window_manager.set_rects(rects);
    }
}
