use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use super::Error;
use super::render_state::{HitTarget, RenderState};
use super::window::Window;
use crate::os_broker::OsBroker;
use crate::rect::PositionedLint;

/// Owns the winit event loop and the overlay windows created for each monitor.
///
/// `WindowManager` is intentionally separate from `Highlighter` because winit event-loop ownership
/// is a process-level concern. It also keeps monitor enumeration, native window lifecycle, and event
/// dispatch out of the public highlighter API.
pub struct WindowManager {
    event_loop: EventLoop<()>,
    context: egui::Context,
    rects: Vec<PositionedLint>,
    os_broker: Box<dyn OsBroker>,
    read_interval: Duration,
}

impl WindowManager {
    pub fn new(
        context: egui::Context,
        os_broker: Box<dyn OsBroker>,
        read_interval: Duration,
    ) -> Result<Self, Error> {
        Ok(Self {
            event_loop: EventLoop::new()?,
            context,
            rects: Vec::new(),
            os_broker,
            read_interval,
        })
    }

    pub fn set_rects(&mut self, rects: Vec<PositionedLint>) {
        self.rects = rects;
    }

    pub fn set_read_interval(&mut self, read_interval: Duration) {
        self.read_interval = read_interval;
    }

    pub fn run_window_for_each_monitor(self) -> Result<(), Error> {
        let mut app =
            WindowManagerApp::new(self.context, self.rects, self.os_broker, self.read_interval);

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
    read_interval: Duration,
    last_read: Instant,
    hovered_lint: Option<usize>,
    cursor_hittest_enabled: bool,
    error: Option<Error>,
}

impl WindowManagerApp {
    fn new(
        context: egui::Context,
        rects: Vec<PositionedLint>,
        os_broker: Box<dyn OsBroker>,
        read_interval: Duration,
    ) -> Self {
        Self {
            context,
            windows: Vec::new(),
            render_state: RenderState::new(rects),
            os_broker,
            read_interval,
            last_read: Instant::now() - read_interval,
            hovered_lint: None,
            cursor_hittest_enabled: false,
            error: None,
        }
    }

    fn read_rect_updates(&mut self) {
        let rects = self.os_broker.get_boxes();
        self.render_state.set_rects(rects);

        for window in &self.windows {
            window.request_redraw();
        }
    }

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

    fn select_hovered_lint(&mut self) {
        self.render_state.set_highlighted_lint(self.hovered_lint);

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

        self.update_cursor_hittest(event_loop);

        event_loop.set_control_flow(ControlFlow::WaitUntil(self.last_read + self.read_interval));
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
        let should_select_hovered_lint = matches!(
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

        if should_select_hovered_lint {
            self.select_hovered_lint();
        }

        if matches!(&event, WindowEvent::CloseRequested) {
            event_loop.exit();
        }
    }
}
