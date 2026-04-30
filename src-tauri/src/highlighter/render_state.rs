use harper_core::linting::{Lint, Suggestion};

use crate::{
    lint_kind_color::lint_kind_color,
    rect::{PositionedLint, Rect},
};

const CARD_WIDTH: f32 = 340.0;

/// Stores highlighter-specific drawing state and renders it into an egui frame.
///
/// `Window` owns the native window and GPU plumbing; `RenderState` owns the content we want to draw
/// into that plumbing. This keeps future highlight rectangles, styling, and animation state out of
/// the platform/rendering infrastructure.
pub struct RenderState {
    /// Lints with their screen-space bounds, used to draw all highlights and resolve hit tests.
    rects: Vec<PositionedLint>,

    /// Index of the lint whose suggestion popup is currently visible.
    ///
    /// `None` means no popup should be rendered.
    highlighted_lint: Option<usize>,
}

impl RenderState {
    pub fn new(rects: Vec<PositionedLint>) -> Self {
        let mut state = Self {
            rects: Vec::new(),
            highlighted_lint: None,
        };
        state.set_rects(rects);
        state
    }

    pub fn set_rects(&mut self, rects: Vec<PositionedLint>) {
        self.rects = rects;

        if self
            .highlighted_lint
            .is_some_and(|index| index >= self.rects.len())
        {
            self.highlighted_lint = None;
        }
    }

    /// Updates which lint owns the suggestion popup without exposing render-state internals.
    ///
    /// Window/input code decides what the user interacted with, while `RenderState` owns the popup
    /// selection. Filtering invalid indexes here keeps stale cursor state from selecting a lint that
    /// disappeared after the latest accessibility read.
    pub fn set_highlighted_lint(&mut self, highlighted_lint: Option<usize>) {
        self.highlighted_lint = highlighted_lint.filter(|index| *index < self.rects.len());
    }

    /// Reports whether a popup is open so window hit-testing can stay enabled for interaction.
    ///
    /// The overlay is normally click-through. Once a popup is visible, the window manager needs a
    /// simple render-state query to avoid making the window click-through while the user is moving
    /// from a highlight into the popup.
    pub fn has_highlighted_lint(&self) -> bool {
        self.highlighted_lint.is_some()
    }

    /// Finds the lint under a screen-space cursor position for hover and click routing.
    ///
    /// Cursor polling lives outside the renderer, but hit-testing belongs next to the rectangles
    /// being rendered so both paths use the same geometry.
    pub fn find_lint_at_pos(&self, pos: egui::Pos2) -> Option<usize> {
        self.rects
            .iter()
            .position(|positioned_lint| rect_bounds(&positioned_lint.rect).contains(pos))
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        for positioned_lint in &self.rects {
            draw_highlight(ui, &positioned_lint.rect, &positioned_lint.lint);
        }

        if let Some(positioned_lint) = self
            .highlighted_lint
            .and_then(|index| self.rects.get(index))
        {
            render_lint_card(ui, &positioned_lint.rect, &positioned_lint.lint);
        }
    }
}

fn draw_highlight(ui: &mut egui::Ui, rect: &Rect, lint: &Lint) {
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

fn render_lint_card(ui: &mut egui::Ui, rect: &Rect, lint: &Lint) {
    egui::Area::new(egui::Id::new("harper-lint-card"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(
            rect.x as f32,
            rect.y as f32 + rect.height as f32 + 8.0,
        ))
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(30, 32, 38, 244))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 34),
                ))
                .corner_radius(egui::CornerRadius::same(14))
                .inner_margin(egui::Margin::same(12))
                .shadow(egui::Shadow {
                    offset: [0, 10],
                    blur: 22,
                    spread: 0,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 90),
                })
                .show(ui, |ui| {
                    ui.set_width(CARD_WIDTH);
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);

                    lint_kind_badge(ui, lint);

                    ui.label(
                        egui::RichText::new(&lint.message)
                            .color(egui::Color32::from_rgb(238, 241, 247))
                            .size(14.0),
                    );

                    ui.add_space(2.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

                        for suggestion in &lint.suggestions {
                            suggestion_option(ui, suggestion);
                        }
                    });
                });
        });
}

fn lint_kind_badge(ui: &mut egui::Ui, lint: &Lint) {
    egui::Frame::new()
        .fill(lint_color(lint))
        .corner_radius(egui::CornerRadius::same(20))
        .inner_margin(egui::Margin::symmetric(9, 4))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(lint.lint_kind.to_string())
                    .strong()
                    .size(12.0)
                    .color(egui::Color32::WHITE),
            );
        });
}

fn suggestion_option(ui: &mut egui::Ui, suggestion: &Suggestion) {
    let button = egui::Button::new(
        egui::RichText::new(suggestion_text(suggestion))
            .size(13.0)
            .color(egui::Color32::from_rgb(246, 248, 252)),
    )
    .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20))
    .stroke(egui::Stroke::new(
        1.0,
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 28),
    ))
    .corner_radius(egui::CornerRadius::same(9));

    ui.add(button);
}

fn suggestion_text(suggestion: &Suggestion) -> String {
    match suggestion {
        Suggestion::ReplaceWith(chars) | Suggestion::InsertAfter(chars) => chars.iter().collect(),
        Suggestion::Remove => "Remove".to_owned(),
    }
}

fn rect_bounds(rect: &Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(rect.x as f32, rect.y as f32),
        egui::vec2(rect.width as f32, rect.height as f32),
    )
}

fn lint_color(lint: &Lint) -> egui::Color32 {
    let color = lint_kind_color(lint.lint_kind);

    egui::Color32::from_rgb(color.r, color.g, color.b)
}
