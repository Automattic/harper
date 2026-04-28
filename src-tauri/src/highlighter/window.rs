use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::monitor::MonitorHandle;
use winit::window::{Window as WinitWindow, WindowButtons, WindowLevel};

use super::Error;

pub struct Window {
    inner: WinitWindow,
}

impl Window {
    pub fn new(event_loop: &ActiveEventLoop, monitor: MonitorHandle) -> Result<Self, Error> {
        let position = monitor.position();
        let size = monitor.size();
        let window = event_loop.create_window(
            WinitWindow::default_attributes()
                .with_title("Harper")
                .with_inner_size(size)
                .with_position(position)
                .with_resizable(false)
                .with_enabled_buttons(WindowButtons::empty())
                .with_decorations(false)
                .with_transparent(true)
                .with_window_level(WindowLevel::AlwaysOnTop)
                .with_active(false),
        )?;

        window.set_outer_position(PhysicalPosition::new(position.x, position.y));
        let _ = window.request_inner_size(PhysicalSize::new(size.width, size.height));
        window.set_cursor_hittest(false)?;

        Ok(Self { inner: window })
    }
}
