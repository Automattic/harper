mod error;
mod render_state;
mod window;
mod window_manager;

use std::collections::BTreeMap;
use std::time::Duration;

pub use error::Error;
use window_manager::{WindowManager, WindowManagerCallbacks, WindowManagerIntervals};

use harper_core::{Document, linting::Lint};

use crate::os_broker::{LintText, OsBroker};
use crate::rect::ActionableLint;

const DEFAULT_READ_INTERVAL: Duration = Duration::from_millis(100);

type IgnoreLint = Box<dyn FnMut(&Lint, &Document)>;
type AddToDictionary = Box<dyn FnMut(&str)>;
type DisableRule = Box<dyn FnMut(&str)>;
type RefreshConfig = Box<dyn FnMut()>;

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
        os_broker: impl OsBroker + 'static,
        lint_text: impl FnMut(&str) -> BTreeMap<String, Vec<Lint>> + 'static,
        ignore_lint: impl FnMut(&Lint, &Document) + 'static,
        add_to_dictionary: impl FnMut(&str) + 'static,
        disable_rule: impl FnMut(&str) + 'static,
        refresh_config: impl FnMut() + 'static,
    ) -> Result<Self, Error> {
        let context = egui::Context::default();
        let lint_text: LintText = Box::new(lint_text);
        let ignore_lint: IgnoreLint = Box::new(ignore_lint);
        let add_to_dictionary: AddToDictionary = Box::new(add_to_dictionary);
        let disable_rule: DisableRule = Box::new(disable_rule);
        let refresh_config: RefreshConfig = Box::new(refresh_config);

        Ok(Self {
            window_manager: WindowManager::new(
                context.clone(),
                Box::new(os_broker),
                WindowManagerCallbacks {
                    lint_text,
                    ignore_lint,
                    add_to_dictionary,
                    disable_rule,
                    refresh_config,
                },
                WindowManagerIntervals {
                    read: DEFAULT_READ_INTERVAL,
                    config_poll: Duration::from_secs(1),
                },
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

    pub fn set_rects(&mut self, rects: Vec<ActionableLint>) {
        self.window_manager.set_rects(rects);
    }
}
