use crate::rect::ColoredRect;

/// Stores highlighter-specific drawing state and renders it into an egui frame.
///
/// `Window` owns the native window and GPU plumbing; `RenderState` owns the content we want to draw
/// into that plumbing. This keeps future highlight rectangles, styling, and animation state out of
/// the platform/rendering infrastructure.
pub struct RenderState {
    rects: Vec<ColoredRect>,
}

impl RenderState {
    pub fn new(rects: Vec<ColoredRect>) -> Self {
        let mut state = Self { rects: Vec::new() };
        state.set_rects(rects);
        state
    }

    pub fn set_rects(&mut self, rects: Vec<ColoredRect>) {
        self.rects = rects;
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        for rect in &self.rects {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.x as f32, rect.y as f32),
                    egui::vec2(rect.width as f32, rect.height as f32),
                ),
                0.0,
                egui::Color32::from_rgba_premultiplied(
                    rect.color.r,
                    rect.color.g,
                    rect.color.b,
                    48,
                ),
            );
        }
    }
}
