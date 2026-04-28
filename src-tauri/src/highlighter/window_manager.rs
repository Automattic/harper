use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use super::Error;
use super::window::Window;

pub(super) struct WindowManager {
    event_loop: EventLoop<()>,
    context: egui::Context,
}

impl WindowManager {
    pub(super) fn new(context: egui::Context) -> Result<Self, Error> {
        Ok(Self {
            event_loop: EventLoop::new()?,
            context,
        })
    }

    pub(super) fn run_window_for_each_monitor(self) -> Result<(), Error> {
        let mut app = WindowManagerApp::new(self.context);

        self.event_loop.set_control_flow(ControlFlow::Wait);
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
    error: Option<Error>,
}

impl WindowManagerApp {
    fn new(context: egui::Context) -> Self {
        Self {
            context,
            windows: Vec::new(),
            error: None,
        }
    }
}

impl ApplicationHandler for WindowManagerApp {
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
                window.render();
            }
        }

        if matches!(event, WindowEvent::CloseRequested) {
            event_loop.exit();
        }
    }
}
