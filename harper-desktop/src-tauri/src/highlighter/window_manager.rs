use std::collections::{HashMap, HashSet};
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
use super::OpenSettings;
use super::RefreshConfig;
use super::render_state::{HitTarget, RenderState};
use super::window::Window;
use crate::os_broker::{LintText, OsBroker};
use crate::rect::ActionableLint;

const OS_POLL_INTERVAL: Duration = Duration::from_millis(16);
const CONFIG_POLL_INTERVAL: Duration = Duration::from_secs(1);
const OVERLAY_RESUME_CHECK_INTERVAL: Duration = Duration::from_millis(500);

/// Owns the winit event loop and the overlay windows created for each monitor.
///
/// `WindowManager` is intentionally separate from `Highlighter` because winit event-loop ownership
/// is a process-level concern. It also keeps monitor enumeration, native window lifecycle, and event
/// dispatch out of the public highlighter API.
pub struct WindowManager {
    event_loop: EventLoop<()>,
    rects: Vec<ActionableLint>,
    os_broker: Box<dyn OsBroker>,
    lint_text: LintText,
    ignore_lint: IgnoreLint,
    add_to_dictionary: AddToDictionary,
    disable_rule: DisableRule,
    refresh_config: RefreshConfig,
    open_settings: OpenSettings,
}

pub struct WindowManagerCallbacks {
    pub lint_text: LintText,
    pub ignore_lint: IgnoreLint,
    pub add_to_dictionary: AddToDictionary,
    pub disable_rule: DisableRule,
    pub refresh_config: RefreshConfig,
    pub open_settings: OpenSettings,
}

impl WindowManager {
    /// Creates the event loop before windows exist because winit requires window creation to happen
    /// from inside that loop's lifecycle callbacks.
    pub fn new(
        os_broker: Box<dyn OsBroker>,
        callbacks: WindowManagerCallbacks,
    ) -> Result<Self, Error> {
        let mut event_loop_builder = EventLoop::builder();

        #[cfg(target_os = "macos")]
        event_loop_builder
            .with_activation_policy(ActivationPolicy::Accessory)
            .with_default_menu(false);

        Ok(Self {
            event_loop: event_loop_builder.build()?,
            rects: Vec::new(),
            os_broker,
            lint_text: callbacks.lint_text,
            ignore_lint: callbacks.ignore_lint,
            add_to_dictionary: callbacks.add_to_dictionary,
            disable_rule: callbacks.disable_rule,
            refresh_config: callbacks.refresh_config,
            open_settings: callbacks.open_settings,
        })
    }

    /// Seeds externally supplied lint rectangles before the event loop takes ownership of rendering.
    pub fn set_rects(&mut self, rects: Vec<ActionableLint>) {
        self.rects = rects;
    }

    /// Transfers ownership into winit's application handler because `run_app` owns the process-level
    /// event loop until the overlay exits.
    pub fn run_window_for_each_monitor(self) -> Result<(), Error> {
        let mut app = WindowManagerApp::new(
            self.rects,
            self.os_broker,
            WindowManagerCallbacks {
                lint_text: self.lint_text,
                ignore_lint: self.ignore_lint,
                add_to_dictionary: self.add_to_dictionary,
                disable_rule: self.disable_rule,
                refresh_config: self.refresh_config,
                open_settings: self.open_settings,
            },
        );

        // Polling and repaint deadlines wake the loop; unchanged overlays need no presentation.
        self.event_loop.set_control_flow(ControlFlow::Wait);
        let result = self.event_loop.run_app(&mut app);

        if let Some(error) = app.error {
            return Err(error);
        }

        result.map_err(Error::from)
    }
}

/// Pending egui repaint deadlines, independent of accessibility polling and native input redraws.
/// Each deadline is consumed once; a rendered frame replaces it with its new repaint request.
#[derive(Default)]
struct RepaintSchedule {
    deadlines: HashMap<WindowId, Instant>,
}

impl RepaintSchedule {
    /// Keeps the earliest request for a window. `Duration::MAX` means no repaint is requested.
    fn request(&mut self, window: WindowId, now: Instant, delay: Duration) {
        if delay == Duration::MAX {
            return;
        }
        if let Some(deadline) = now.checked_add(delay) {
            self.deadlines
                .entry(window)
                .and_modify(|pending| *pending = (*pending).min(deadline))
                .or_insert(deadline);
        }
    }

    fn cancel(&mut self, window: WindowId) {
        self.deadlines.remove(&window);
    }

    /// Consumes an expired deadline so requesting a redraw does not itself keep waking the loop.
    fn take_due(&mut self, window: WindowId, now: Instant) -> bool {
        if self
            .deadlines
            .get(&window)
            .is_some_and(|deadline| *deadline <= now)
        {
            self.cancel(window);
            true
        } else {
            false
        }
    }

    fn next_deadline(&self) -> Option<Instant> {
        self.deadlines.values().copied().min()
    }
}

struct WindowManagerApp {
    windows: Vec<Window>,
    render_state: RenderState,
    os_broker: Box<dyn OsBroker>,
    lint_text: LintText,
    last_config_poll: Instant,
    next_os_poll: Instant,
    repaints: RepaintSchedule,
    refresh_config: RefreshConfig,
    hovered_lint: Option<usize>,
    cursor_hittest_enabled: bool,
    occluded_windows: HashSet<WindowId>,
    overlays_paused: bool,
    error: Option<Error>,
}

impl WindowManagerApp {
    /// Builds the mutable application state consumed by winit callbacks after `WindowManager` gives
    /// up direct control of the event loop.
    fn new(
        rects: Vec<ActionableLint>,
        os_broker: Box<dyn OsBroker>,
        callbacks: WindowManagerCallbacks,
    ) -> Self {
        Self {
            windows: Vec::new(),
            render_state: RenderState::new(
                rects,
                callbacks.ignore_lint,
                callbacks.add_to_dictionary,
                callbacks.disable_rule,
                callbacks.open_settings,
            ),
            os_broker,
            lint_text: callbacks.lint_text,
            last_config_poll: Instant::now(),
            next_os_poll: Instant::now(),
            repaints: RepaintSchedule::default(),
            refresh_config: callbacks.refresh_config,
            hovered_lint: None,
            cursor_hittest_enabled: false,
            occluded_windows: HashSet::new(),
            overlays_paused: false,
            error: None,
        }
    }

    /// Refreshes lint geometry from the OS broker inside the event loop so repaint requests happen on
    /// the same thread that owns the overlay windows.
    fn read_rect_updates(&mut self) {
        if let Some(lints) = self.os_broker.get_boxes(self.lint_text.as_mut())
            && self.render_state.set_lints(lints)
        {
            self.request_redraws();
        }
    }

    fn request_redraws(&self) {
        for window in &self.windows {
            if self.can_render_window(window.id()) {
                window.request_redraw();
            }
        }
    }

    /// Whether to suspend overlay OS reads and rendering because no overlay can be seen.
    /// Configuration synchronization remains active. An empty window list alone does not imply
    /// occlusion: windows may not have been created yet.
    fn should_pause_overlays(&self) -> bool {
        all_displays_asleep()
            || (!self.windows.is_empty()
                && self
                    .windows
                    .iter()
                    .all(|window| self.occluded_windows.contains(&window.id())))
    }

    /// Checks redraw eligibility using the pause state last refreshed in `about_to_wait` and the
    /// latest window occlusion events. Apply this both when requesting and consuming redraws:
    /// an already-queued redraw must not render an unavailable overlay.
    fn can_render_window(&self, window_id: WindowId) -> bool {
        !self.overlays_paused && !self.occluded_windows.contains(&window_id)
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
        self.request_redraws();
    }
}

impl ApplicationHandler for WindowManagerApp {
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if now.duration_since(self.last_config_poll) >= CONFIG_POLL_INTERVAL {
            self.refresh_config();
            self.last_config_poll = Instant::now();
        }

        if now >= self.next_os_poll {
            // Rendering while displays sleep can rapidly accumulate Metal drawables (#3874).
            // Pause OS reads and repaint deadlines, but keep configuration synchronization alive.
            let was_paused = self.overlays_paused;
            self.overlays_paused = self.should_pause_overlays();
            let interval = if self.overlays_paused {
                self.repaints.deadlines.clear();
                OVERLAY_RESUME_CHECK_INTERVAL
            } else {
                self.read_rect_updates();
                self.update_cursor_hittest(event_loop);
                if was_paused {
                    self.request_redraws();
                }
                OS_POLL_INTERVAL
            };
            // Schedule after the work finishes: slow accessibility reads must not create a busy loop.
            self.next_os_poll = Instant::now() + interval;
        }

        let now = Instant::now();
        for window in &self.windows {
            if self.can_render_window(window.id()) && self.repaints.take_due(window.id(), now) {
                window.request_redraw();
            }
        }

        let mut wake_at = self
            .next_os_poll
            .min(self.last_config_poll + CONFIG_POLL_INTERVAL);
        if let Some(repaint_at) = self.repaints.next_deadline() {
            wake_at = wake_at.min(repaint_at);
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(wake_at));
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.windows.is_empty() {
            return;
        }

        let monitors = event_loop.available_monitors().collect::<Vec<_>>();

        for monitor in monitors {
            match pollster::block_on(Window::new(event_loop, monitor)) {
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
        if let WindowEvent::Occluded(occluded) = &event {
            if *occluded {
                self.occluded_windows.insert(window_id);
                self.repaints.cancel(window_id);
            } else {
                self.occluded_windows.remove(&window_id);
                self.repaints
                    .request(window_id, Instant::now(), Duration::ZERO);
            }
            self.next_os_poll = Instant::now();
        }

        // Ignore queued redraws while paused: no GPU work or new animation deadlines should
        // be created for an overlay that cannot be presented (#3874).
        if matches!(&event, WindowEvent::RedrawRequested) && !self.can_render_window(window_id) {
            return;
        }

        let should_select_hit_lint = matches!(
            &event,
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            }
        );
        let should_render = matches!(&event, WindowEvent::RedrawRequested);
        let screen_rects = if should_render {
            self.windows.iter().map(Window::screen_rect).collect()
        } else {
            Vec::new()
        };

        if let Some((index, window)) = self
            .windows
            .iter_mut()
            .enumerate()
            .find(|(_, window)| window.id() == window_id)
        {
            window.handle_event(&event);

            if should_render {
                self.repaints.cancel(window_id);
                let repaint_delay = window.render(&mut self.render_state, &screen_rects, index);
                self.repaints
                    .request(window_id, Instant::now(), repaint_delay);
            }
        }

        if should_select_hit_lint && !self.overlays_paused {
            self.select_hit_lint();
        }

        if matches!(&event, WindowEvent::CloseRequested) {
            event_loop.exit();
        }
    }
}

/// Whether macOS reports no awake, drawable displays. Sleeping displays may disappear from the
/// active list, so an empty list also pauses rendering. On query failure, leave rendering enabled
/// rather than indefinitely hiding highlights.
#[cfg(target_os = "macos")]
fn all_displays_asleep() -> bool {
    use core_graphics::display::CGDisplay;

    CGDisplay::active_displays()
        .map(|displays| {
            displays
                .into_iter()
                .all(|id| CGDisplay::new(id).is_asleep())
        })
        .unwrap_or(false)
}

#[cfg(not(target_os = "macos"))]
fn all_displays_asleep() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_schedule_has_no_deadline() {
        let mut schedule = RepaintSchedule::default();
        let now = Instant::now();
        let window = WindowId::from(1);
        schedule.request(window, now, Duration::MAX);
        assert_eq!(schedule.next_deadline(), None);
        assert!(!schedule.take_due(window, now));
    }

    #[test]
    fn deadline_is_consumed_once_when_due() {
        let mut schedule = RepaintSchedule::default();
        let now = Instant::now();
        let window = WindowId::from(1);
        let deadline = now + OS_POLL_INTERVAL;
        schedule.request(window, now, OS_POLL_INTERVAL);
        assert_eq!(schedule.next_deadline(), Some(deadline));
        assert!(!schedule.take_due(window, now));
        assert!(schedule.take_due(window, deadline));
        assert!(!schedule.take_due(window, deadline));
        assert_eq!(schedule.next_deadline(), None);
    }

    #[test]
    fn immediate_repaint_does_not_persist() {
        let mut schedule = RepaintSchedule::default();
        let now = Instant::now();
        let window = WindowId::from(1);
        schedule.request(window, now, Duration::ZERO);
        assert!(schedule.take_due(window, now));
        assert_eq!(schedule.next_deadline(), None);
    }

    #[test]
    fn earliest_request_wins_without_losing_other_windows() {
        let mut schedule = RepaintSchedule::default();
        let now = Instant::now();
        let first = WindowId::from(1);
        let second = WindowId::from(2);
        schedule.request(first, now, Duration::from_millis(30));
        schedule.request(first, now, Duration::from_millis(10));
        schedule.request(first, now, Duration::from_millis(50));
        schedule.request(first, now, Duration::MAX);
        schedule.request(second, now, Duration::from_millis(20));
        assert_eq!(
            schedule.next_deadline(),
            Some(now + Duration::from_millis(10))
        );
        assert!(schedule.take_due(first, now + Duration::from_millis(10)));
        assert!(!schedule.take_due(second, now + Duration::from_millis(10)));
        assert_eq!(
            schedule.next_deadline(),
            Some(now + Duration::from_millis(20))
        );
    }

    #[test]
    fn input_redraw_can_replace_an_old_animation_deadline() {
        let mut schedule = RepaintSchedule::default();
        let now = Instant::now();
        let window = WindowId::from(1);
        schedule.request(window, now, OS_POLL_INTERVAL);
        // An input-triggered frame was rendered before its scheduled deadline and is now idle.
        schedule.cancel(window);
        schedule.request(window, now, Duration::MAX);
        assert_eq!(schedule.next_deadline(), None);
    }

    #[test]
    fn occlusion_cancels_only_that_windows_repaint() {
        let mut schedule = RepaintSchedule::default();
        let now = Instant::now();
        let hidden = WindowId::from(1);
        let visible = WindowId::from(2);
        schedule.request(hidden, now, Duration::ZERO);
        schedule.request(visible, now, OS_POLL_INTERVAL);
        schedule.cancel(hidden);
        assert!(!schedule.take_due(hidden, now));
        assert_eq!(schedule.next_deadline(), Some(now + OS_POLL_INTERVAL));
        assert!(schedule.take_due(visible, now + OS_POLL_INTERVAL));
    }
}
