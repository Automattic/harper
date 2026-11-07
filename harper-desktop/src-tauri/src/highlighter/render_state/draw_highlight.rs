//! Draws lint highlights without owning popup selection or application state.

use harper_core::linting::Lint;

use super::{lint_kind_color32, rect_bounds};
use crate::rect::Rect;

/// Draws the always-visible lint marker without making the renderer responsible for popup state.
pub fn draw_highlight(ui: &mut egui::Ui, rect: &Rect, lint: &Lint) {
    let anim_id = egui::Id::new("lint-highlight-anim")
        .with(rect.x.to_bits())
        .with(rect.y.to_bits())
        .with(rect.width.to_bits())
        .with(rect.height.to_bits());
    let anim_progress = ui.ctx().animate_bool_with_time(anim_id, true, 0.20);
    let t = 1.0 - (1.0 - anim_progress).powi(2);

    let rect_bounds = rect_bounds(rect);
    let color = lint_color(lint);
    let [r, g, b, _] = color.to_array();
    let fill_alpha = (24.0 * t) as u8;
    let underline_alpha = (255.0 * t) as u8;
    let fill_color = egui::Color32::from_rgba_unmultiplied(r, g, b, fill_alpha);
    let underline_color = egui::Color32::from_rgba_unmultiplied(r, g, b, underline_alpha);
    let underline_height = rect_bounds.height().min(2.0);

    let animated_right = rect_bounds.left() + rect_bounds.width() * t;
    let animated_bounds = egui::Rect::from_min_max(
        rect_bounds.left_top(),
        egui::pos2(animated_right, rect_bounds.bottom()),
    );

    ui.painter().rect_filled(animated_bounds, 0.0, fill_color);
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect_bounds.left(), rect_bounds.bottom() - underline_height),
            egui::pos2(animated_right, rect_bounds.bottom()),
        ),
        0.0,
        underline_color,
    );
}

/// Maps Harper lint kinds to egui colors at the drawing boundary so shared color data stays UI-toolkit
/// agnostic.
fn lint_color(lint: &Lint) -> egui::Color32 {
    lint_kind_color32(lint.lint_kind)
}
