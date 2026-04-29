use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use super::Error;
use super::ReadRects;
use super::render_state::RenderState;
use super::window::Window;
use crate::rect::ColoredRect;

/// Owns the winit event loop and the overlay windows created for each monitor.
///
/// `WindowManager` is intentionally separate from `Highlighter` because winit event-loop ownership
/// is a process-level concern. It also keeps monitor enumeration, native window lifecycle, and event
/// dispatch out of the public highlighter API.
pub struct WindowManager {
    event_loop: EventLoop<()>,
    context: egui::Context,
    rects: Vec<ColoredRect>,
    read_rects: ReadRects,
    read_interval: Duration,
}

impl WindowManager {
    pub fn new(
        context: egui::Context,
        read_rects: ReadRects,
        read_interval: Duration,
    ) -> Result<Self, Error> {
        Ok(Self {
            event_loop: EventLoop::new()?,
            context,
            rects: Vec::new(),
            read_rects,
            read_interval,
        })
    }

    pub fn set_rects(&mut self, rects: Vec<ColoredRect>) {
        self.rects = rects;
    }

    pub fn set_read_interval(&mut self, read_interval: Duration) {
        self.read_interval = read_interval;
    }

    pub fn run_window_for_each_monitor(self) -> Result<(), Error> {
        let mut app = WindowManagerApp::new(
            self.context,
            self.rects,
            self.read_rects,
            self.read_interval,
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
    read_rects: ReadRects,
    read_interval: Duration,
    last_read: Instant,
    error: Option<Error>,
}

impl WindowManagerApp {
    fn new(
        context: egui::Context,
        rects: Vec<ColoredRect>,
        read_rects: ReadRects,
        read_interval: Duration,
    ) -> Self {
        Self {
            context,
            windows: Vec::new(),
            render_state: RenderState::new(rects),
            read_rects,
            read_interval,
            last_read: Instant::now() - read_interval,
            error: None,
        }
    }

    fn read_rect_updates(&mut self) {
        if let Some(rects) = (self.read_rects)() {
            self.render_state.set_rects(rects);

            for window in &self.windows {
                window.request_redraw();
            }
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
        if let Some(window) = self
            .windows
            .iter_mut()
            .find(|window| window.id() == window_id)
        {
            window.handle_event(&event);
            if matches!(event, WindowEvent::RedrawRequested) {
                window.render(&mut self.render_state);
            }
        }

        if matches!(event, WindowEvent::CloseRequested) {
            event_loop.exit();
        }
    }
}
