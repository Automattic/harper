use crate::rect::ColoredRect;

const CARD_WIDTH: f32 = 340.0;

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
        let example = ExampleLint {
            x: 100.0,
            y: 100.0,
            width: 128.0,
            height: 24.0,
            color: egui::Color32::from_rgb(238, 66, 102),
            lint_kind: "Spelling",
            message: "Possible spelling mistake found in this text.",
            suggestions: &[
                "Replace with: \"suggestions\"",
                "Replace with: \"suggestion\"",
                "Ignore",
            ],
        };

        draw_highlight(ui, &example);
        render_lint_card(ui, &example);
    }
}

struct ExampleLint {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: egui::Color32,
    lint_kind: &'static str,
    message: &'static str,
    suggestions: &'static [&'static str],
}

fn draw_highlight(ui: &mut egui::Ui, lint: &ExampleLint) {
    let rect = lint_bounds(lint);
    let [r, g, b, _] = lint.color.to_array();
    let fill_color = egui::Color32::from_rgba_unmultiplied(r, g, b, 24);
    let underline_color = egui::Color32::from_rgba_unmultiplied(r, g, b, 255);
    let underline_height = rect.height().min(2.0);

    ui.painter().rect_filled(rect, 0.0, fill_color);
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.left(), rect.bottom() - underline_height),
            rect.right_bottom(),
        ),
        0.0,
        underline_color,
    );
}

fn render_lint_card(ui: &mut egui::Ui, lint: &ExampleLint) {
    egui::Area::new(egui::Id::new("harper-lint-card-example"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(lint.x, lint.y + lint.height + 8.0))
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
                        egui::RichText::new(lint.message)
                            .color(egui::Color32::from_rgb(238, 241, 247))
                            .size(14.0),
                    );

                    ui.add_space(2.0);
                    for suggestion in lint.suggestions {
                        suggestion_option(ui, suggestion);
                    }
                });
        });
}

fn lint_kind_badge(ui: &mut egui::Ui, lint: &ExampleLint) {
    egui::Frame::new()
        .fill(lint.color)
        .corner_radius(egui::CornerRadius::same(20))
        .inner_margin(egui::Margin::symmetric(9, 4))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(lint.lint_kind)
                    .strong()
                    .size(12.0)
                    .color(egui::Color32::WHITE),
            );
        });
}

fn suggestion_option(ui: &mut egui::Ui, suggestion: &str) {
    let button = egui::Button::new(
        egui::RichText::new(suggestion)
            .size(13.0)
            .color(egui::Color32::from_rgb(246, 248, 252)),
    )
    .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20))
    .stroke(egui::Stroke::new(
        1.0,
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 28),
    ))
    .corner_radius(egui::CornerRadius::same(9));

    ui.add_sized([CARD_WIDTH, 34.0], button);
}

fn lint_bounds(lint: &ExampleLint) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(lint.x, lint.y),
        egui::vec2(lint.width, lint.height),
    )
}
