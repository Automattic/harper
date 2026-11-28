//! Draws lint highlights without owning popup selection or application state.

use harper_core::linting::Lint;

use super::{lint_kind_color32, rect_bounds};
use crate::rect::Rect;

/// Draws the always-visible lint marker without making the renderer responsible for popup state.
pub fn draw_highlight(ui: &mut egui::Ui, rect: &Rect, lint: &Lint) {
    let rect_bounds = rect_bounds(rect);
    let color = lint_color(lint);
    let [r, g, b, _] = color.to_array();
    let fill_color = egui::Color32::from_rgba_unmultiplied(r, g, b, 24);
    let underline_color = egui::Color32::from_rgba_unmultiplied(r, g, b, 255);
    let underline_height = rect_bounds.height().min(2.0);

    ui.painter().rect_filled(rect_bounds, 0.0, fill_color);
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect_bounds.left(), rect_bounds.bottom() - underline_height),
            rect_bounds.right_bottom(),
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
