use std::num::NonZeroU32;
use std::sync::Arc;

use egui_wgpu::wgpu::PresentMode;
use egui_wgpu::winit::Painter;
use egui_wgpu::{RendererOptions, WgpuConfiguration, WgpuSetup};
use winit::dpi::{PhysicalPosition, PhysicalSize, Position, Size};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::monitor::MonitorHandle;
use winit::window::{Window as WinitWindow, WindowButtons, WindowId, WindowLevel};

use super::Error;
use super::render_state::RenderState;

/// A transparent click-through overlay window for one monitor.
///
/// `Window` owns the native winit window plus egui/wgpu integration required to render into it. It
/// deliberately does not own highlighter drawing decisions; those are supplied by `RenderState`
/// during each redraw.
///
/// Each window owns its own egui context. A painter only receives the texture updates (such as
/// font atlas changes) from passes run on its own window, so windows that share one context end
/// up drawing with incomplete textures.
pub struct Window {
    inner: Arc<WinitWindow>,
    egui_state: egui_winit::State,
    painter: Painter,
    viewport_id: egui::ViewportId,
    /// The monitor this window covers, in the global screen space of lint rectangles.
    bounds: egui::Rect,
}

/// Computes a monitor's bounds in the global screen space of lint rectangles and cursor positions.
///
/// Those coordinates are in logical points, and each monitor's origin is scaled by that monitor's
/// own factor.
fn monitor_bounds(
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
    scale_factor: f64,
) -> egui::Rect {
    let position = position.to_logical::<f32>(scale_factor);
    let size = size.to_logical::<f32>(scale_factor);

    egui::Rect::from_min_size(
        egui::pos2(position.x, position.y),
        egui::vec2(size.width, size.height),
    )
}

/// Selects the units for placing an overlay window on `monitor`.
///
/// On macOS, winit converts physical values with the scale of the main screen or of the window's
/// current screen, not of the target monitor. With monitors of different scales, physical values can
/// misplace the window, so macOS uses logical values, which are not converted.
///
/// On Windows, physical values are exact, and logical values would have the same problem.
fn overlay_geometry(monitor: &MonitorHandle) -> (Position, Size) {
    let position = monitor.position();
    let size = monitor.size();

    if cfg!(target_os = "macos") {
        let scale_factor = monitor.scale_factor();

        (
            position.to_logical::<f64>(scale_factor).into(),
            size.to_logical::<f64>(scale_factor).into(),
        )
    } else {
        (position.into(), size.into())
    }
}

impl Window {
    pub async fn new(event_loop: &ActiveEventLoop, monitor: MonitorHandle) -> Result<Self, Error> {
        let context = egui::Context::default();
        let bounds = monitor_bounds(monitor.position(), monitor.size(), monitor.scale_factor());
        let (position, size) = overlay_geometry(&monitor);
        let window = Arc::new(
            event_loop.create_window(
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
            )?,
        );

        window.set_outer_position(position);
        let _ = window.request_inner_size(size);
        window.set_cursor_hittest(false)?;
        let viewport_id = egui::ViewportId::from_hash_of(window.id());

        let egui_state = egui_winit::State::new(
            context.clone(),
            viewport_id,
            event_loop,
            Some(window.scale_factor() as f32),
            window.theme(),
            None,
        );

        let mut painter = Painter::new(
            context,
            WgpuConfiguration {
                present_mode: PresentMode::Fifo,
                wgpu_setup: WgpuSetup::from_display_handle(event_loop.owned_display_handle()),
                ..Default::default()
            },
            true,
            RendererOptions::default(),
        )
        .await;
        painter
            .set_window(viewport_id, Some(window.clone()))
            .await?;
        window.request_redraw();

        Ok(Self {
            inner: window,
            egui_state,
            painter,
            viewport_id,
            bounds,
        })
    }

    pub fn id(&self) -> WindowId {
        self.inner.id()
    }

    pub fn bounds(&self) -> egui::Rect {
        self.bounds
    }

    pub fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    /// Controls whether the transparent overlay can receive pointer events.
    ///
    /// Highlight windows are click-through by default so the underlying app remains usable. The
    /// window manager temporarily enables hit-testing when global cursor polling shows the pointer
    /// is over an interactive highlight or popup.
    pub fn set_cursor_hittest(&self, enabled: bool) -> Result<(), Error> {
        self.inner.set_cursor_hittest(enabled)?;

        Ok(())
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        let response = self.egui_state.on_window_event(&self.inner, event);

        if response.repaint {
            self.inner.request_redraw();
        }

        if let WindowEvent::Resized(size) = event
            && let (Some(width), Some(height)) =
                (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        {
            self.painter
                .on_window_resized(self.viewport_id, width, height);
            self.inner.request_redraw();
        }
    }

    pub fn render(&mut self, render_state: &mut RenderState) {
        let context = self.egui_state.egui_ctx().clone();
        let input = self.egui_state.take_egui_input(&self.inner);
        let output = context.run_ui(input, |ui| {
            render_state.render(ui, self.bounds);
        });

        self.egui_state
            .handle_platform_output(&self.inner, output.platform_output);

        let clipped_primitives = context.tessellate(output.shapes, output.pixels_per_point);
        self.painter.paint_and_update_textures(
            self.viewport_id,
            output.pixels_per_point,
            [0.0, 0.0, 0.0, 0.0],
            &clipped_primitives,
            &output.textures_delta,
            Vec::new(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_bounds_use_each_monitors_own_scale() {
        let retina = monitor_bounds(
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(3456, 2234),
            2.0,
        );
        let external = monitor_bounds(
            PhysicalPosition::new(1728, -200),
            PhysicalSize::new(1920, 1080),
            1.0,
        );

        assert_eq!(
            retina,
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1728.0, 1117.0))
        );
        assert_eq!(
            external,
            egui::Rect::from_min_size(egui::pos2(1728.0, -200.0), egui::vec2(1920.0, 1080.0))
        );
    }
}
