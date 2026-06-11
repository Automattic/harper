use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use harper_core::{
    Document,
    linting::{Lint, LintKind, Suggestion},
};

use super::{AddToDictionary, DisableRule, IgnoreLint};
use crate::{
    lint_kind_color::lint_kind_color,
    rect::{ActionableLint, Rect},
};

const CARD_WIDTH: f32 = 440.0;
const CARD_HEIGHT: f32 = 178.0;
const CARD_OFFSET_Y: f32 = 8.0;
const HEADER_HEIGHT: f32 = 46.0;
const FOOTER_HEIGHT: f32 = 43.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTarget {
    Lint(usize),
    Popup,
    None,
}

enum LintCardAction {
    Close,
    ApplySuggestion(Suggestion),
    IgnoreLint,
    AddToDictionary,
    DisableRule,
}

#[derive(Debug, Clone, Copy)]
struct PopupStyle {
    color: egui::Color32,
    background: egui::Color32,
    foreground: egui::Color32,
}

#[derive(Debug, Clone, Copy)]
enum Glyph {
    Close,
    Settings,
    Disable,
    Plus,
}

/// Stores highlighter-specific drawing state and renders it into an egui frame.
///
/// `Window` owns the native window and GPU plumbing; `RenderState` owns the content we want to draw
/// into that plumbing. This keeps future highlight rectangles, styling, and animation state out of
/// the platform/rendering infrastructure.
pub struct RenderState {
    /// Lints with their screen-space bounds, used to draw all highlights and resolve hit tests.
    rects: Vec<ActionableLint>,

    /// Index of the lint whose suggestion popup is currently visible.
    ///
    /// `None` means no popup should be rendered.
    highlighted_lint: Option<usize>,

    /// Cache for markdown-rendered lint messages so popup redraws do not repeatedly rebuild markdown
    /// resources from scratch.
    markdown_cache: CommonMarkCache,

    /// Called when the user dismisses a lint so app-level ignored-lint state can be updated.
    ignore_lint: IgnoreLint,

    /// Called when the user adds a spelling lint's source text to the local dictionary.
    add_to_dictionary: AddToDictionary,

    /// Called when the user disables the rule that produced the selected lint.
    disable_rule: DisableRule,
}

impl RenderState {
    /// Creates render state through `set_rects` so initial lint data gets the same stale-popup guard
    /// used by later accessibility refreshes.
    pub fn new(
        rects: Vec<ActionableLint>,
        ignore_lint: IgnoreLint,
        add_to_dictionary: AddToDictionary,
        disable_rule: DisableRule,
    ) -> Self {
        let mut state = Self {
            rects: Vec::new(),
            highlighted_lint: None,
            markdown_cache: CommonMarkCache::default(),
            ignore_lint,
            add_to_dictionary,
            disable_rule,
        };
        state.set_rects(rects);
        state
    }

    /// Replaces the accessibility-derived lint geometry while preventing a popup from pointing at a
    /// lint index that no longer exists after the latest read.
    pub fn set_rects(&mut self, rects: Vec<ActionableLint>) {
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

    /// Centralizes the close behavior so the popup close button only has to clear the selected lint.
    pub fn close_popup(&mut self) {
        self.highlighted_lint = None;
    }

    /// Finds the interactive highlighter region under a screen-space cursor position.
    ///
    /// Cursor polling lives outside the renderer, but hit-testing belongs next to the rectangles and
    /// popup geometry being rendered so both paths use the same layout contract.
    pub fn hit_target_at_pos(&self, pos: egui::Pos2) -> HitTarget {
        if self.popup_rect().is_some_and(|rect| rect.contains(pos)) {
            return HitTarget::Popup;
        }

        self.rects
            .iter()
            .position(|positioned_lint| rect_bounds(&positioned_lint.rect).contains(pos))
            .map_or(HitTarget::None, HitTarget::Lint)
    }

    /// Computes popup hit-test bounds from our layout contract instead of waiting for egui to report
    /// rendered bounds, which keeps hit-testing available before the next render pass completes.
    pub fn popup_rect(&self) -> Option<egui::Rect> {
        self.highlighted_lint
            .and_then(|index| self.rects.get(index))
            .map(|positioned_lint| popup_rect_for_lint(&positioned_lint.rect))
    }

    /// Draws highlights and the active popup from the same state used by hit-testing so visible
    /// regions and clickable regions do not drift apart.
    pub fn render(&mut self, ui: &mut egui::Ui) {
        for positioned_lint in &self.rects {
            draw_highlight(ui, &positioned_lint.rect, &positioned_lint.lint);
        }

        if let Some(index) = self.highlighted_lint
            && let Some(positioned_lint) = self.rects.get(index)
        {
            let rect = positioned_lint.rect;
            let lint = positioned_lint.lint.clone();
            let source_text = positioned_lint.source_text.clone();

            match render_lint_card(ui, &rect, &lint, &source_text, &mut self.markdown_cache) {
                Some(LintCardAction::Close) => self.close_popup(),
                Some(LintCardAction::ApplySuggestion(suggestion)) => {
                    if let Some(actionable_lint) = self.rects.get_mut(index) {
                        actionable_lint.apply_suggestion(suggestion);
                    }

                    self.close_popup();
                }
                Some(LintCardAction::IgnoreLint) => {
                    if let Some((lint, source_text)) =
                        self.rects.get(index).map(|actionable_lint| {
                            (
                                actionable_lint.lint.clone(),
                                actionable_lint.source_text.clone(),
                            )
                        })
                    {
                        let document = Document::new_markdown_default_curated(&source_text);
                        (self.ignore_lint)(&lint, &document);
                    }

                    self.close_popup();
                }
                Some(LintCardAction::AddToDictionary) => {
                    if let Some((lint, source_text)) =
                        self.rects.get(index).map(|actionable_lint| {
                            (
                                actionable_lint.lint.clone(),
                                actionable_lint.source_text.clone(),
                            )
                        })
                    {
                        let document = Document::new_markdown_default_curated(&source_text);
                        let word = lint.get_str(document.get_source());
                        (self.add_to_dictionary)(&word);
                    }

                    self.close_popup();
                }
                Some(LintCardAction::DisableRule) => {
                    if let Some(rule_name) = self
                        .rects
                        .get(index)
                        .map(|actionable_lint| actionable_lint.rule_name.clone())
                    {
                        (self.disable_rule)(&rule_name);
                    }

                    self.close_popup();
                }
                None => {}
            }
        }
    }
}

/// Draws the always-visible lint marker without making the renderer responsible for popup state.
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

/// Renders the suggestion popup and returns whether the explicit close control was clicked.
fn render_lint_card(
    ui: &mut egui::Ui,
    rect: &Rect,
    lint: &Lint,
    source_text: &str,
    markdown_cache: &mut CommonMarkCache,
) -> Option<LintCardAction> {
    let popup_rect = popup_rect_for_lint(rect);

    egui::Area::new(egui::Id::new("harper-lint-card"))
        .order(egui::Order::Foreground)
        .fixed_pos(popup_rect.min)
        .show(ui.ctx(), |ui| {
            let mut action = None;

            egui::Frame::new()
                .fill(hex(0xff, 0xfd, 0xfa))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 20),
                ))
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::same(0))
                .shadow(egui::Shadow {
                    offset: [0, 14],
                    blur: 32,
                    spread: 0,
                    color: egui::Color32::from_rgba_unmultiplied(20, 12, 2, 56),
                })
                .show(ui, |ui| {
                    ui.set_width(CARD_WIDTH);
                    ui.set_min_height(CARD_HEIGHT);
                    ui.set_max_height(CARD_HEIGHT);

                    render_popover_header(ui, lint, &mut action);
                    render_popover_body(ui, lint, markdown_cache, &mut action);
                    render_popover_footer(ui, lint, source_text, &mut action);
                });
            action
        })
        .inner
}

/// Renders the top bar from the prototype, including no-op utility icons and the real close action.
fn render_popover_header(ui: &mut egui::Ui, lint: &Lint, action: &mut Option<LintCardAction>) {
    let style = popup_style_for_lint_kind(lint.lint_kind);

    egui::Frame::new()
        .fill(blend(style.background, hex(0xff, 0xfd, 0xfa), 0.42))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 15),
        ))
        .corner_radius(egui::CornerRadius {
            nw: 12,
            ne: 12,
            sw: 0,
            se: 0,
        })
        .inner_margin(egui::Margin::symmetric(10, 10))
        .show(ui, |ui| {
            ui.set_width(CARD_WIDTH - 20.0);
            ui.set_height(HEADER_HEIGHT - 20.0);
            ui.horizontal(|ui| {
                lint_kind_badge(ui, lint, style);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icon_button(ui, Glyph::Close, "Close this suggestion popup.").clicked() {
                        *action = Some(LintCardAction::Close);
                    }
                    icon_button(ui, Glyph::Settings, "Open Harper settings.");
                    if icon_button(ui, Glyph::Disable, "Disable this rule.").clicked() {
                        *action = Some(LintCardAction::DisableRule);
                    }
                });
            });
        });
}

/// Renders the markdown explanation and horizontal suggestion chip row from the prototype body.
fn render_popover_body(
    ui: &mut egui::Ui,
    lint: &Lint,
    markdown_cache: &mut CommonMarkCache,
    action: &mut Option<LintCardAction>,
) {
    egui::Frame::new()
        .fill(hex(0xff, 0xfd, 0xfa))
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.set_width(CARD_WIDTH - 32.0);
            ui.set_min_height(CARD_HEIGHT - HEADER_HEIGHT - FOOTER_HEIGHT - 24.0);
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            render_lint_message(ui, markdown_cache, &lint.message);

            if !lint.suggestions.is_empty() {
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

                    for (index, suggestion) in lint.suggestions.iter().enumerate() {
                        if suggestion_option(ui, lint.lint_kind, suggestion, index == 0).clicked() {
                            *action = Some(LintCardAction::ApplySuggestion(suggestion.clone()));
                        }
                    }
                });
            }
        });
}

/// Renders the footer controls as visual stubs so the popup matches the target component before those
/// actions have real behavior.
fn render_popover_footer(
    ui: &mut egui::Ui,
    lint: &Lint,
    source_text: &str,
    action: &mut Option<LintCardAction>,
) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 5))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 15),
        ))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(CARD_WIDTH - 20.0);
            ui.set_height(FOOTER_HEIGHT - 16.0);
            ui.horizontal(|ui| {
                if is_spelling_kind(lint.lint_kind) {
                    let source_chars = source_text.chars().collect::<Vec<_>>();
                    let word = lint.get_str(&source_chars);
                    if ghost_button(
                        ui,
                        Some(Glyph::Plus),
                        "Add to dictionary",
                        format!("Add \"{word}\" to your personal dictionary."),
                    )
                    .clicked()
                    {
                        *action = Some(LintCardAction::AddToDictionary);
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ghost_button(ui, None, "Dismiss", "Ignore this suggestion").clicked() {
                        *action = Some(LintCardAction::IgnoreLint);
                    }
                });
            });
        });
}

/// Renders Harper's markdown-formatted lint message in the popup instead of showing raw markdown
/// markers to the user.
fn render_lint_message(ui: &mut egui::Ui, cache: &mut CommonMarkCache, message: &str) {
    ui.scope(|ui| {
        ui.visuals_mut().code_bg_color = hex(0xf3, 0xf4, 0xf6);
        ui.visuals_mut().extreme_bg_color = hex(0xf8, 0xfa, 0xfc);
        ui.visuals_mut().override_text_color = Some(hex(0x37, 0x41, 0x51));
        ui.visuals_mut().text_edit_bg_color = Some(hex(0xf8, 0xfa, 0xfc));
        CommonMarkViewer::new()
            .default_width(Some(ui.available_width() as usize))
            .show(ui, cache, message);
    });
}

/// Keeps the lint type visually attached to the popup without duplicating lint-kind formatting in
/// the window manager.
fn lint_kind_badge(ui: &mut egui::Ui, lint: &Lint, style: PopupStyle) {
    egui::Frame::new()
        .fill(style.background)
        .corner_radius(egui::CornerRadius::same(20))
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);
                let (dot_rect, _) =
                    ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(dot_rect.center(), 3.0, style.color);
                ui.label(
                    egui::RichText::new(lint.lint_kind.to_string().to_uppercase())
                        .strong()
                        .size(11.0)
                        .color(style.foreground),
                );
            });
        });
}

/// Renders a suggestion as a compact button so later replacement behavior can attach to one place.
fn suggestion_option(
    ui: &mut egui::Ui,
    lint_kind: LintKind,
    suggestion: &Suggestion,
    primary: bool,
) -> egui::Response {
    let lint_color = lint_kind_color32(lint_kind);
    let fill = if primary {
        lint_color
    } else {
        blend(lint_color, hex(0xff, 0xfd, 0xfa), 0.88)
    };
    let text_color = if primary {
        hex(0xff, 0xfd, 0xfa)
    } else {
        hex(0x0a, 0x0a, 0x0a)
    };
    let stroke = if primary {
        egui::Stroke::NONE
    } else {
        egui::Stroke::new(1.0, blend(lint_color, hex(0xff, 0xfd, 0xfa), 0.64))
    };

    ui.scope(|ui| {
        ui.spacing_mut().button_padding = egui::vec2(16.0, 10.0);
        let stroke_width = stroke.width;
        ui.visuals_mut().widgets.inactive.expansion = 0.0;
        ui.visuals_mut().widgets.hovered.expansion = 0.0;
        ui.visuals_mut().widgets.active.expansion = 0.0;
        ui.visuals_mut().widgets.inactive.bg_stroke.width = stroke_width;
        ui.visuals_mut().widgets.hovered.bg_stroke.width = stroke_width;
        ui.visuals_mut().widgets.active.bg_stroke.width = stroke_width;

        let button = egui::Button::new(
            egui::RichText::new(suggestion_text(suggestion))
                .size(13.5)
                .color(text_color),
        )
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(8))
        .min_size(egui::vec2(0.0, 38.0));

        hover_text(ui.add(button), suggestion_hover_text(suggestion))
    })
    .inner
}

/// Renders the prototype's square icon-only controls without coupling their visual treatment to any
/// behavior beyond the response returned to the caller.
fn icon_button(ui: &mut egui::Ui, glyph: Glyph, text: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::click());
    let background = if response.hovered() {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 15)
    } else {
        egui::Color32::TRANSPARENT
    };
    let color = if response.hovered() {
        hex(0x00, 0x00, 0x00)
    } else {
        hex(0x6b, 0x72, 0x80)
    };

    ui.painter().rect_filled(rect, 6.0, background);
    draw_glyph(ui, rect.shrink(6.0), glyph, color);

    hover_text(response, text)
}

/// Renders footer controls as no-op buttons so future actions can be wired without changing layout.
fn ghost_button(
    ui: &mut egui::Ui,
    glyph: Option<Glyph>,
    label: &str,
    text: impl Into<String>,
) -> egui::Response {
    let label = if glyph.is_some() {
        format!("+ {label}")
    } else {
        label.to_string()
    };

    ui.scope(|ui| {
        ui.spacing_mut().button_padding = egui::vec2(8.0, 6.0);
        let response = ui.add(
            egui::Button::new(
                egui::RichText::new(label)
                    .size(12.0)
                    .color(hex(0x4b, 0x55, 0x63)),
            )
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
            .corner_radius(egui::CornerRadius::same(6)),
        );

        hover_text(response, text)
    })
    .inner
}

/// Applies tooltip styling that visually belongs with the suggestion popup instead of egui defaults.
fn hover_text(response: egui::Response, hover_text: impl Into<String>) -> egui::Response {
    let hover_text = hover_text.into();

    let mut tooltip = egui::Tooltip::for_enabled(&response);
    tooltip.popup = tooltip.popup.frame(
        egui::Frame::new()
            .fill(hex(0xff, 0xfd, 0xfa))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 20),
            ))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(8, 6))
            .shadow(egui::Shadow {
                offset: [0, 8],
                blur: 18,
                spread: 0,
                color: egui::Color32::from_rgba_unmultiplied(20, 12, 2, 42),
            }),
    );
    tooltip.show(|ui| {
        ui.set_max_width(260.0);
        ui.label(
            egui::RichText::new(hover_text)
                .size(12.0)
                .color(hex(0x4b, 0x55, 0x63)),
        );
    });

    response
}

/// Draws the small SVG-inspired line icons from the prototype using egui painter primitives.
fn draw_glyph(ui: &egui::Ui, rect: egui::Rect, glyph: Glyph, color: egui::Color32) {
    match glyph {
        Glyph::Close => draw_close_icon(ui, rect, color),
        Glyph::Settings => draw_settings_icon(ui, rect, color),
        Glyph::Disable => draw_disable_icon(ui, rect, color),
        Glyph::Plus => draw_plus_icon(ui, rect, color),
    }
}

/// Draws the close glyph separately from button behavior so icon styling can change without touching
/// the popup's interaction contract.
fn draw_close_icon(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.6, color);

    ui.painter()
        .line_segment([rect.left_top(), rect.right_bottom()], stroke);
    ui.painter()
        .line_segment([rect.right_top(), rect.left_bottom()], stroke);
}

fn draw_settings_icon(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.5, color);
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.32;

    ui.painter().circle_stroke(center, radius, stroke);
    ui.painter().circle_stroke(center, radius * 0.38, stroke);

    for angle in [0.0_f32, 60.0, 120.0, 180.0, 240.0, 300.0] {
        let radians = angle.to_radians();
        let dir = egui::vec2(radians.cos(), radians.sin());
        ui.painter().line_segment(
            [center + dir * radius, center + dir * (radius + 2.8)],
            stroke,
        );
    }
}

fn draw_disable_icon(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.5, color);

    ui.painter().circle_stroke(
        rect.center(),
        rect.width().min(rect.height()) * 0.45,
        stroke,
    );
    ui.painter()
        .line_segment([rect.left_bottom(), rect.right_top()], stroke);
}

fn draw_plus_icon(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.6, color);

    ui.painter().line_segment(
        [
            egui::pos2(rect.center().x, rect.top()),
            egui::pos2(rect.center().x, rect.bottom()),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            egui::pos2(rect.left(), rect.center().y),
            egui::pos2(rect.right(), rect.center().y),
        ],
        stroke,
    );
}

fn is_spelling_kind(lint_kind: LintKind) -> bool {
    match lint_kind {
        LintKind::Eggcorn | LintKind::Malapropism | LintKind::Spelling | LintKind::Typo => true,
        LintKind::Agreement
        | LintKind::BoundaryError
        | LintKind::Capitalization
        | LintKind::Enhancement
        | LintKind::Formatting
        | LintKind::Grammar
        | LintKind::Miscellaneous
        | LintKind::Nonstandard
        | LintKind::Preposition
        | LintKind::Punctuation
        | LintKind::Readability
        | LintKind::Redundancy
        | LintKind::Regionalism
        | LintKind::Repetition
        | LintKind::Style
        | LintKind::Usage
        | LintKind::WordChoice => false,
    }
}

fn popup_style_for_lint_kind(lint_kind: LintKind) -> PopupStyle {
    let color = lint_kind_color32(lint_kind);

    match lint_kind {
        LintKind::Eggcorn | LintKind::Malapropism | LintKind::Spelling | LintKind::Typo => {
            PopupStyle {
                color,
                background: hex(0xff, 0xee, 0xf2),
                foreground: hex(0xbe, 0x12, 0x3c),
            }
        }
        LintKind::Agreement
        | LintKind::BoundaryError
        | LintKind::Grammar
        | LintKind::Preposition
        | LintKind::Repetition => PopupStyle {
            color,
            background: hex(0xff, 0xf7, 0xed),
            foreground: hex(0xc2, 0x41, 0x0c),
        },
        LintKind::Capitalization => PopupStyle {
            color,
            background: hex(0xf3, 0xee, 0xff),
            foreground: hex(0x5b, 0x21, 0xb6),
        },
        LintKind::Formatting | LintKind::Punctuation | LintKind::Readability => PopupStyle {
            color,
            background: hex(0xef, 0xf6, 0xff),
            foreground: hex(0x1d, 0x4e, 0xd8),
        },
        LintKind::Enhancement
        | LintKind::Miscellaneous
        | LintKind::Nonstandard
        | LintKind::Redundancy
        | LintKind::Regionalism
        | LintKind::Style
        | LintKind::Usage
        | LintKind::WordChoice => PopupStyle {
            color,
            background: hex(0xf3, 0xf4, 0xf6),
            foreground: hex(0x37, 0x41, 0x51),
        },
    }
}

/// Converts Harper suggestion variants into button text.
fn suggestion_text(suggestion: &Suggestion) -> String {
    match suggestion {
        Suggestion::ReplaceWith(chars) | Suggestion::InsertAfter(chars) => chars.iter().collect(),
        Suggestion::Remove => "Remove".to_owned(),
    }
}

fn suggestion_hover_text(suggestion: &Suggestion) -> String {
    match suggestion {
        Suggestion::Remove => "Remove".to_owned(),
        Suggestion::ReplaceWith(chars) | Suggestion::InsertAfter(chars) => {
            format!("Replace with \"{}\"", chars.iter().collect::<String>())
        }
    }
}

/// Converts accessibility rectangles into egui rectangles so rendering and hit-testing use the same
/// screen-space geometry.
fn rect_bounds(rect: &Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(rect.x as f32, rect.y as f32),
        egui::vec2(rect.width as f32, rect.height as f32),
    )
}

/// Defines popup geometry ahead of rendering so the transparent overlay can enable or disable native
/// hit-testing based on our intended layout, not a previous frame's measured egui output.
fn popup_rect_for_lint(rect: &Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(
            rect.x as f32,
            rect.y as f32 + rect.height as f32 + CARD_OFFSET_Y,
        ),
        egui::vec2(CARD_WIDTH, CARD_HEIGHT),
    )
}

/// Maps Harper lint kinds to egui colors at the drawing boundary so shared color data stays UI-toolkit
/// agnostic.
fn lint_color(lint: &Lint) -> egui::Color32 {
    lint_kind_color32(lint.lint_kind)
}

fn lint_kind_color32(lint_kind: LintKind) -> egui::Color32 {
    let color = lint_kind_color(lint_kind);

    egui::Color32::from_rgb(color.r, color.g, color.b)
}

fn hex(r: u8, g: u8, b: u8) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

fn blend(from: egui::Color32, to: egui::Color32, to_weight: f32) -> egui::Color32 {
    let from_weight = 1.0 - to_weight;
    let [fr, fg, fb, _] = from.to_array();
    let [tr, tg, tb, _] = to.to_array();

    egui::Color32::from_rgb(
        ((f32::from(fr) * from_weight) + (f32::from(tr) * to_weight)) as u8,
        ((f32::from(fg) * from_weight) + (f32::from(tg) * to_weight)) as u8,
        ((f32::from(fb) * from_weight) + (f32::from(tb) * to_weight)) as u8,
    )
}
