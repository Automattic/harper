use std::num::NonZeroU32;
use std::sync::Arc;

use egui_wgpu::wgpu::PresentMode;
use egui_wgpu::winit::Painter;
use egui_wgpu::{RendererOptions, WgpuConfiguration, WgpuSetup};
use winit::dpi::{PhysicalPosition, PhysicalSize};
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
}

impl Window {
    pub async fn new(event_loop: &ActiveEventLoop, monitor: MonitorHandle) -> Result<Self, Error> {
        let context = egui::Context::default();
        let position = monitor.position();
        let size = monitor.size();
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

        window.set_outer_position(PhysicalPosition::new(position.x, position.y));
        let _ = window.request_inner_size(PhysicalSize::new(size.width, size.height));
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
        })
    }

    pub fn id(&self) -> WindowId {
        self.inner.id()
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

    /// The window's bounds in the screen coordinates that lints and the cursor use.
    ///
    /// On platforms that cannot report where the window is, this assumes the window is at the
    /// screen's origin.
    pub fn screen_rect(&self) -> egui::Rect {
        let pixels_per_point =
            egui_winit::pixels_per_point(self.egui_state.egui_ctx(), &self.inner);

        egui_winit::inner_rect_in_points(&self.inner, pixels_per_point).unwrap_or_else(|| {
            let size = self.inner.inner_size();
            egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(size.width as f32, size.height as f32) / pixels_per_point,
            )
        })
    }

    /// Draws `render_state` into this window, which is `screen_rects[index]`.
    pub fn render(
        &mut self,
        render_state: &mut RenderState,
        screen_rects: &[egui::Rect],
        index: usize,
    ) {
        let context = self.egui_state.egui_ctx().clone();
        let input = self.egui_state.take_egui_input(&self.inner);
        let output = context.run_ui(input, |ui| {
            render_state.render(ui, screen_rects, index);
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
