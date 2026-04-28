use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use super::Error;
use super::window::Window;

pub(super) struct WindowManager {
    event_loop: EventLoop<()>,
}

impl WindowManager {
    pub(super) fn new() -> Result<Self, Error> {
        Ok(Self {
            event_loop: EventLoop::new()?,
        })
    }

    pub(super) fn run_window_for_each_monitor(self) -> Result<(), Error> {
        let mut app = WindowManagerApp::default();

        self.event_loop.set_control_flow(ControlFlow::Wait);
        let result = self.event_loop.run_app(&mut app);

        if let Some(error) = app.error {
            return Err(error);
        }

        result.map_err(Error::from)
    }
}

#[derive(Default)]
struct WindowManagerApp {
    windows: Vec<Window>,
    error: Option<Error>,
}

impl ApplicationHandler for WindowManagerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.windows.is_empty() {
            return;
        }

        for monitor in event_loop.available_monitors() {
            match Window::new(event_loop, monitor) {
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
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if matches!(event, WindowEvent::CloseRequested) {
            event_loop.exit();
        }
    }
}
