use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
use winit::window::WindowId;

use super::AddToDictionary;
use super::DisableRule;
use super::Error;
use super::IgnoreLint;
use super::RefreshConfig;
use super::render_state::{HitTarget, RenderState};
use super::window::Window;
use crate::os_broker::{LintText, OsBroker};
use crate::rect::ActionableLint;

/// Owns the winit event loop and the overlay windows created for each monitor.
///
/// `WindowManager` is intentionally separate from `Highlighter` because winit event-loop ownership
/// is a process-level concern. It also keeps monitor enumeration, native window lifecycle, and event
/// dispatch out of the public highlighter API.
pub struct WindowManager {
    event_loop: EventLoop<()>,
    context: egui::Context,
    rects: Vec<ActionableLint>,
    os_broker: Box<dyn OsBroker>,
    lint_text: LintText,
    ignore_lint: IgnoreLint,
    add_to_dictionary: AddToDictionary,
    disable_rule: DisableRule,
    refresh_config: RefreshConfig,
    read_interval: Duration,
    config_poll_interval: Duration,
}

pub struct WindowManagerCallbacks {
    pub lint_text: LintText,
    pub ignore_lint: IgnoreLint,
    pub add_to_dictionary: AddToDictionary,
    pub disable_rule: DisableRule,
    pub refresh_config: RefreshConfig,
}

pub struct WindowManagerIntervals {
    pub read: Duration,
    pub config_poll: Duration,
}

impl WindowManager {
    /// Creates the event loop before windows exist because winit requires window creation to happen
    /// from inside that loop's lifecycle callbacks.
    pub fn new(
        context: egui::Context,
        os_broker: Box<dyn OsBroker>,
        callbacks: WindowManagerCallbacks,
        intervals: WindowManagerIntervals,
    ) -> Result<Self, Error> {
        let mut event_loop_builder = EventLoop::builder();

        #[cfg(target_os = "macos")]
        event_loop_builder
            .with_activation_policy(ActivationPolicy::Accessory)
            .with_default_menu(false);

        Ok(Self {
            event_loop: event_loop_builder.build()?,
            context,
            rects: Vec::new(),
            os_broker,
            lint_text: callbacks.lint_text,
            ignore_lint: callbacks.ignore_lint,
            add_to_dictionary: callbacks.add_to_dictionary,
            disable_rule: callbacks.disable_rule,
            refresh_config: callbacks.refresh_config,
            read_interval: intervals.read,
            config_poll_interval: intervals.config_poll,
        })
    }

    /// Seeds externally supplied lint rectangles before the event loop takes ownership of rendering.
    pub fn set_rects(&mut self, rects: Vec<ActionableLint>) {
        self.rects = rects;
    }

    /// Lets callers tune OS polling without exposing the window-manager internals that schedule it.
    pub fn set_read_interval(&mut self, read_interval: Duration) {
        self.read_interval = read_interval;
    }

    /// Transfers ownership into winit's application handler because `run_app` owns the process-level
    /// event loop until the overlay exits.
    pub fn run_window_for_each_monitor(self) -> Result<(), Error> {
        let mut app = WindowManagerApp::new(
            self.context,
            self.rects,
            self.os_broker,
            WindowManagerCallbacks {
                lint_text: self.lint_text,
                ignore_lint: self.ignore_lint,
                add_to_dictionary: self.add_to_dictionary,
                disable_rule: self.disable_rule,
                refresh_config: self.refresh_config,
            },
            WindowManagerIntervals {
                read: self.read_interval,
                config_poll: self.config_poll_interval,
            },
        );

        self.event_loop
            .set_control_flow(ControlFlow::WaitUntil(Instant::now() + self.read_interval));
        let result = self.event_loop.run_app(&mut app);

        if let Some(error) = app.error {
            return Err(error);
        }

        result.map_err(Error::from)
    }
}

struct WindowManagerApp {
    context: egui::Context,
    windows: Vec<Window>,
    render_state: RenderState,
    os_broker: Box<dyn OsBroker>,
    lint_text: LintText,
    read_interval: Duration,
    config_poll_interval: Duration,
    last_read: Instant,
    last_config_poll: Instant,
    refresh_config: RefreshConfig,
    hovered_lint: Option<usize>,
    cursor_hittest_enabled: bool,
    error: Option<Error>,
}

impl WindowManagerApp {
    /// Builds the mutable application state consumed by winit callbacks after `WindowManager` gives
    /// up direct control of the event loop.
    fn new(
        context: egui::Context,
        rects: Vec<ActionableLint>,
        os_broker: Box<dyn OsBroker>,
        callbacks: WindowManagerCallbacks,
        intervals: WindowManagerIntervals,
    ) -> Self {
        let read_interval = intervals.read;
        Self {
            context,
            windows: Vec::new(),
            render_state: RenderState::new(
                rects,
                callbacks.ignore_lint,
                callbacks.add_to_dictionary,
                callbacks.disable_rule,
            ),
            os_broker,
            lint_text: callbacks.lint_text,
            read_interval,
            config_poll_interval: intervals.config_poll,
            last_read: Instant::now() - read_interval,
            last_config_poll: Instant::now(),
            refresh_config: callbacks.refresh_config,
            hovered_lint: None,
            cursor_hittest_enabled: false,
            error: None,
        }
    }

    /// Refreshes lint geometry from the OS broker inside the event loop so repaint requests happen on
    /// the same thread that owns the overlay windows.
    fn read_rect_updates(&mut self) {
        let rects = self.os_broker.get_boxes(self.lint_text.as_mut());
        self.render_state.set_rects(rects);

        for window in &self.windows {
            window.request_redraw();
        }
    }

    fn refresh_config(&mut self) {
        (self.refresh_config)();
    }

    /// Toggles native click-through behavior based on the cursor's current interactive Harper target,
    /// keeping editor clicks from being swallowed when the pointer is outside highlights and popups.
    fn update_cursor_hittest(&mut self, event_loop: &ActiveEventLoop) {
        let Some(cursor_pos) = self.os_broker.cursor_position() else {
            return;
        };

        let hit_target = self.render_state.hit_target_at_pos(cursor_pos);
        self.hovered_lint = match hit_target {
            HitTarget::Lint(index) => Some(index),
            HitTarget::Popup | HitTarget::None => None,
        };

        let should_enable_hittest = !matches!(hit_target, HitTarget::None);

        if self.cursor_hittest_enabled == should_enable_hittest {
            return;
        }

        for window in &self.windows {
            if let Err(error) = window.set_cursor_hittest(should_enable_hittest) {
                self.error = Some(error);
                event_loop.exit();
                return;
            }
        }

        self.cursor_hittest_enabled = should_enable_hittest;
    }

    /// Opens or replaces the popup only when the actual click target is a lint, leaving popup clicks
    /// for egui controls such as the close button.
    fn select_hit_lint(&mut self) {
        let Some(cursor_pos) = self.os_broker.cursor_position() else {
            return;
        };

        let HitTarget::Lint(index) = self.render_state.hit_target_at_pos(cursor_pos) else {
            return;
        };

        self.render_state.set_highlighted_lint(Some(index));

        for window in &self.windows {
            window.request_redraw();
        }
    }
}

impl ApplicationHandler for WindowManagerApp {
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if now.duration_since(self.last_read) >= self.read_interval {
            self.read_rect_updates();
            self.last_read = now;
        }

        if now.duration_since(self.last_config_poll) >= self.config_poll_interval {
            self.refresh_config();
            self.last_config_poll = now;
        }

        self.update_cursor_hittest(event_loop);

        let next_read = self.last_read + self.read_interval;
        let next_config_poll = self.last_config_poll + self.config_poll_interval;
        event_loop.set_control_flow(ControlFlow::WaitUntil(next_read.min(next_config_poll)));
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.windows.is_empty() {
            return;
        }

        for monitor in event_loop.available_monitors() {
            match pollster::block_on(Window::new(event_loop, monitor, self.context.clone())) {
                Ok(window) => self.windows.push(window),
                Err(error) => {
                    self.error = Some(error);
                    event_loop.exit();
                    return;
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let should_select_hit_lint = matches!(
            &event,
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            }
        );
        let should_render = matches!(&event, WindowEvent::RedrawRequested);

        if let Some(window) = self
            .windows
            .iter_mut()
            .find(|window| window.id() == window_id)
        {
            window.handle_event(&event);

            if should_render {
                window.render(&mut self.render_state);
            }
        }

        if should_select_hit_lint {
            self.select_hit_lint();
        }

        if matches!(&event, WindowEvent::CloseRequested) {
            event_loop.exit();
        }
    }
}
