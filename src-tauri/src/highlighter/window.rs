use std::num::NonZeroU32;
use std::sync::Arc;

use egui_wgpu::winit::Painter;
use egui_wgpu::{RendererOptions, WgpuConfiguration, WgpuSetup};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::monitor::MonitorHandle;
use winit::window::{Window as WinitWindow, WindowButtons, WindowId, WindowLevel};

use super::Error;

pub(super) struct Window {
    inner: Arc<WinitWindow>,
    egui_state: egui_winit::State,
    painter: Painter,
    viewport_id: egui::ViewportId,
}

impl Window {
    pub(super) async fn new(
        event_loop: &ActiveEventLoop,
        monitor: MonitorHandle,
        context: egui::Context,
    ) -> Result<Self, Error> {
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

    pub(super) fn id(&self) -> WindowId {
        self.inner.id()
    }

    pub(super) fn handle_event(&mut self, event: &WindowEvent) {
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

    pub(super) fn render(&mut self) {
        let context = self.egui_state.egui_ctx().clone();
        let input = self.egui_state.take_egui_input(&self.inner);
        let output = context.run_ui(input, |ui| {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(egui::pos2(150.0, 150.0), egui::vec2(200.0, 200.0)),
                0.0,
                egui::Color32::from_rgba_premultiplied(255, 255, 0, 96),
            );
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
