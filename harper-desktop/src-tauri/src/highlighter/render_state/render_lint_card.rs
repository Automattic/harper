//! Renders the lint card and defines its shared drawing and hit-test geometry.

use std::hash::Hash;

use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use harper_core::linting::{Lint, LintKind, Suggestion};

use super::lint_kind_color32;
use super::popup_layout::{PopupLayout, UNBOUNDED_VIEWPORT, popup_layout};
use crate::rect::Rect;

const LIST_WIDTH: f32 = 210.0;
const FLYOUT_WIDTH: f32 = 190.0;
const DIVIDER_WIDTH: f32 = 1.0;
const BORDER_WIDTH: f32 = 1.0;
const LIST_CARD_WIDTH: f32 = LIST_WIDTH + 2.0 * BORDER_WIDTH;
const FLYOUT_CARD_WIDTH: f32 = FLYOUT_WIDTH + 2.0 * BORDER_WIDTH;
/// The maximum height of either card. Content that does not fit scrolls.
const CARD_HEIGHT: f32 = 400.0;
const CARD_CORNER_RADIUS: u8 = 8;
const ROW_HEIGHT: f32 = 30.0;
const LIST_PADDING_Y: f32 = 4.0;
const FOOTER_ROW_HEIGHT: f32 = 28.0;
/// The space above the footer, the footer divider, the space below it, and the footer row.
const LIST_FOOTER_HEIGHT: f32 = 4.0 + DIVIDER_WIDTH + 4.0 + FOOTER_ROW_HEIGHT;
/// The distance from the list card's top to the first suggestion row's top.
///
/// This is also the sidebar's starting position, so a new popup opens with the sidebar aligned to
/// the first row and does not slide.
pub const FIRST_ROW_OFFSET: f32 = BORDER_WIDTH + LIST_PADDING_Y;
const KIND_HEADER_HEIGHT: f32 = 34.0;
const DISABLE_ROW_HEIGHT: f32 = 33.0;
const BUTTON_MIN_HEIGHT: f32 = 24.0;

const POPUP_BACKGROUND: egui::Color32 = egui::Color32::from_rgb(0xff, 0xff, 0xff);
const POPUP_BORDER: egui::Color32 = egui::Color32::from_rgb(0xd0, 0xd7, 0xde);
const POPUP_SHADOW: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(140, 149, 159, 77);
const DIVIDER_COLOR: egui::Color32 = egui::Color32::from_rgb(0xea, 0xee, 0xf2);
const HOVER_BACKGROUND: egui::Color32 = egui::Color32::from_black_alpha(11);
const TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(0x1f, 0x23, 0x28);
const MUTED_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(0x57, 0x60, 0x6a);
const SUBTLE_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(0x8b, 0x94, 0x9e);
const DISMISS_BACKGROUND: egui::Color32 = egui::Color32::from_rgb(0xe5, 0xe5, 0xe5);
const BUTTON_BACKGROUND: egui::Color32 = egui::Color32::from_rgb(0xef, 0xef, 0xef);
const BUTTON_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00);

/// An action selected in the lint card, returned for `RenderState` to apply.
///
/// The renderer reports intent without updating application state or invoking callbacks.
pub enum LintCardAction {
    Close,
    OpenSettings,
    /// The pointer is on the suggestion row at this index. The flyout card aligns with that row.
    ///
    /// When the lint has no suggestions, index `0` is the "No replacements" row.
    HoverSuggestion(usize),
    ApplySuggestion(Suggestion),
    IgnoreLint,
    AddToDictionary,
    DisableRule,
}

#[derive(Debug, Clone, Copy)]
enum Glyph {
    Sliders,
    BookDown,
}

/// The visible content of a [`flyout_button`].
#[derive(Debug, Clone, Copy)]
enum ButtonContent<'a> {
    Text(&'a str),
    Glyph(Glyph),
}

/// Renders the suggestion popup and returns the action the user picked, if any.
///
/// The layout copies the Chrome extension's suggestion popup: the suggestion list and the lint
/// explanation are two separate floating cards. [`popup_layout_for_lint`] places them, so
/// hit-testing and drawing agree without measuring the popup.
///
/// `sidebar_position` is the flyout card's current offset from the list card's top. The caller
/// animates it toward [`sidebar_target`]. `shown_suggestion` only selects the highlighted row.
pub fn render_lint_card(
    ui: &mut egui::Ui,
    rect: &Rect,
    lint: &Lint,
    source_text: &str,
    shown_suggestion: Option<usize>,
    sidebar_position: f32,
    markdown_cache: &mut CommonMarkCache,
) -> Option<LintCardAction> {
    let layout = popup_layout_for_lint(rect, lint, sidebar_position);
    let problem_text = problem_text(lint, source_text);
    let span = (lint.span.start, lint.span.end);

    let list_action = show_card(ui.ctx(), "harper-lint-card-list", span, layout.list, |ui| {
        let mut action = None;
        render_suggestion_list(
            ui,
            lint,
            &problem_text,
            shown_suggestion.unwrap_or(0),
            &mut action,
        );
        action
    });

    let flyout_action = show_card(
        ui.ctx(),
        "harper-lint-card-flyout",
        span,
        layout.flyout,
        |ui| {
            let mut action = None;
            render_flyout(ui, lint, markdown_cache, &mut action);
            action
        },
    );

    list_action.or(flyout_action)
}

/// Renders one floating card at `rect.min`, with content that scrolls when it is taller than
/// `rect`.
///
/// `rect` is the card's maximum bounds. The card shrinks to fit shorter content, but its available
/// space is reset each frame so a previous short card cannot constrain a longer message.
fn show_card<R>(
    ctx: &egui::Context,
    name: &'static str,
    id_salt: impl Hash,
    rect: egui::Rect,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Area::new(egui::Id::new(name))
        .order(egui::Order::Foreground)
        .fixed_pos(rect.min)
        .show(ctx, |ui| {
            ui.set_max_size(rect.size());
            card_frame()
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::ZERO;

                    egui::ScrollArea::vertical()
                        .id_salt((name, id_salt))
                        .max_height(rect.height() - 2.0 * BORDER_WIDTH)
                        .show(ui, |ui| {
                            ui.set_width(rect.width() - 2.0 * BORDER_WIDTH);
                            add_contents(ui)
                        })
                        .inner
                })
                .inner
        })
        .inner
}

/// The background, border, rounded corners, and shadow shared by both popup cards and tooltips,
/// like the Chrome extension's `.harper-container` and `.harper-flyout`.
fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(POPUP_BACKGROUND)
        .stroke(egui::Stroke::new(BORDER_WIDTH, POPUP_BORDER))
        .corner_radius(egui::CornerRadius::same(CARD_CORNER_RADIUS))
        .shadow(egui::Shadow {
            offset: [0, 4],
            blur: 12,
            spread: 0,
            color: POPUP_SHADOW,
        })
}

/// Renders the list card: one row for each suggestion, then the footer.
///
/// Clicking a row applies its suggestion. Hovering a row reports its index, so the flyout card can
/// align with it. When the lint has no suggestions, one "No replacements" row takes their place,
/// as in the Chrome extension. Clicking that row does nothing.
///
/// The row at `selected` keeps the hover background, like the Chrome extension's
/// `.harper-selected` row.
fn render_suggestion_list(
    ui: &mut egui::Ui,
    lint: &Lint,
    problem_text: &str,
    selected: usize,
    action: &mut Option<LintCardAction>,
) {
    ui.add_space(LIST_PADDING_Y);

    if lint.suggestions.is_empty()
        && suggestion_row(
            ui,
            lint.lint_kind,
            "No replacements",
            format!("No replacements for \"{problem_text}\""),
            selected == 0,
        )
        .hovered()
    {
        *action = Some(LintCardAction::HoverSuggestion(0));
    }

    for (index, suggestion) in lint.suggestions.iter().enumerate() {
        let row = suggestion_row(
            ui,
            lint.lint_kind,
            &suggestion_text(suggestion),
            suggestion_hover_text(suggestion, problem_text),
            selected == index,
        );

        if row.clicked() {
            *action = Some(LintCardAction::ApplySuggestion(suggestion.clone()));
        } else if row.hovered() {
            *action = Some(LintCardAction::HoverSuggestion(index));
        }
    }

    render_list_footer(ui, action);
    ui.add_space(LIST_PADDING_Y);
}

/// Renders one suggestion row: a swatch in the lint color, the label, and a chevron.
///
/// The row's rectangle is allocated before anything is painted, so the hover background can come
/// from this frame's response. A `selected` row gets the same background without the hover.
fn suggestion_row(
    ui: &mut egui::Ui,
    lint_kind: LintKind,
    label: &str,
    hover: impl Into<String>,
    selected: bool,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(LIST_WIDTH, ROW_HEIGHT), egui::Sense::click());
    let painter = ui.painter();

    if selected || response.hovered() {
        painter.rect_filled(rect, 0.0, HOVER_BACKGROUND);
    }

    let swatch = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 10.0, rect.center().y - 8.0),
        egui::vec2(3.0, 16.0),
    );
    painter.rect_filled(swatch, 2.0, lint_kind_color32(lint_kind));

    let chevron = single_line_galley(ui, "›", 12.0, f32::INFINITY);
    let chevron_left = rect.right() - 8.0 - chevron.size().x;
    let label_left = swatch.right() + 8.0;
    let label = single_line_galley(ui, label, 13.0, chevron_left - 8.0 - label_left);

    paint_text(painter, label_left, rect.center().y, label, TEXT_COLOR);
    paint_text(
        painter,
        chevron_left,
        rect.center().y,
        chevron,
        SUBTLE_TEXT_COLOR,
    );

    hover_text(
        response.on_hover_cursor(egui::CursorIcon::PointingHand),
        hover,
    )
}

/// Renders the list footer: a "Go to Harper settings" row and the close button.
///
/// The close button sits over the footer's right edge, where
/// the footer leaves 26 points of padding for it.
fn render_list_footer(ui: &mut egui::Ui, action: &mut Option<LintCardAction>) {
    ui.add_space(4.0);

    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(LIST_WIDTH, DIVIDER_WIDTH + 4.0 + FOOTER_ROW_HEIGHT),
        egui::Sense::hover(),
    );
    ui.painter().hline(
        rect.x_range(),
        rect.top() + DIVIDER_WIDTH / 2.0,
        egui::Stroke::new(DIVIDER_WIDTH, DIVIDER_COLOR),
    );

    let settings_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left(), rect.bottom() - FOOTER_ROW_HEIGHT),
        egui::pos2(rect.right() - 26.0, rect.bottom()),
    );
    let settings = ui.interact(
        settings_rect,
        ui.id().with("harper-lint-card-settings"),
        egui::Sense::click(),
    );

    if settings.hovered() {
        ui.painter()
            .rect_filled(settings_rect, 0.0, HOVER_BACKGROUND);
    }

    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(settings_rect.left() + 10.0, settings_rect.center().y - 7.0),
        egui::vec2(14.0, 14.0),
    );
    draw_glyph(ui, icon_rect, Glyph::Sliders, MUTED_TEXT_COLOR);

    let label_left = icon_rect.right() + 6.0;
    let label = single_line_galley(
        ui,
        "Go to Harper settings",
        12.0,
        settings_rect.right() - 10.0 - label_left,
    );
    paint_text(
        ui.painter(),
        label_left,
        settings_rect.center().y,
        label,
        MUTED_TEXT_COLOR,
    );

    if hover_text(
        settings.on_hover_cursor(egui::CursorIcon::PointingHand),
        "Click to go to Harper settings",
    )
    .clicked()
    {
        *action = Some(LintCardAction::OpenSettings);
    }

    let close_label = single_line_galley(ui, "×", 20.0, f32::INFINITY);
    let close_size = egui::vec2(close_label.size().x + 8.0, 20.0);
    let close_rect = egui::Rect::from_min_size(
        egui::pos2(
            rect.right() - 4.0 - close_size.x,
            rect.top() + DIVIDER_WIDTH + 6.0,
        ),
        close_size,
    );
    let close = ui.interact(
        close_rect,
        ui.id().with("harper-lint-card-close"),
        egui::Sense::click(),
    );
    let close_color = if close.hovered() {
        TEXT_COLOR
    } else {
        MUTED_TEXT_COLOR
    };
    paint_text(
        ui.painter(),
        close_rect.center().x - close_label.size().x / 2.0,
        close_rect.center().y,
        close_label,
        close_color,
    );

    if hover_text(
        close.on_hover_cursor(egui::CursorIcon::PointingHand),
        "Close",
    )
    .clicked()
    {
        *action = Some(LintCardAction::Close);
    }
}

/// Renders the flyout card: the lint kind, the explanation, and the actions for the whole lint.
///
/// The content does not depend on which suggestion row is hovered. Only the card's position does,
/// and [`popup_layout_for_lint`] controls that.
fn render_flyout(
    ui: &mut egui::Ui,
    lint: &Lint,
    markdown_cache: &mut CommonMarkCache,
    action: &mut Option<LintCardAction>,
) {
    render_kind_header(ui, lint.lint_kind);

    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(FLYOUT_WIDTH - 20.0);
            render_lint_message(ui, markdown_cache, &lint.message, lint.lint_kind);
        });

    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 10,
            right: 10,
            top: 0,
            bottom: 8,
        })
        .show(ui, |ui| {
            ui.set_width(FLYOUT_WIDTH - 20.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

                if flyout_button(
                    ui,
                    ButtonContent::Text("Dismiss"),
                    DISMISS_BACKGROUND,
                    "Ignore this lint",
                )
                .clicked()
                {
                    *action = Some(LintCardAction::IgnoreLint);
                }

                if is_spelling_kind(lint.lint_kind)
                    && flyout_button(
                        ui,
                        ButtonContent::Glyph(Glyph::BookDown),
                        BUTTON_BACKGROUND,
                        "Add word to user dictionary",
                    )
                    .clicked()
                {
                    *action = Some(LintCardAction::AddToDictionary);
                }
            });
        });

    if disable_rule_row(ui).clicked() {
        *action = Some(LintCardAction::DisableRule);
    }
}

/// Renders the lint kind above the explanation, underlined in the lint color.
fn render_kind_header(ui: &mut egui::Ui, lint_kind: LintKind) {
    const BORDER_HEIGHT: f32 = 2.0;

    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(FLYOUT_WIDTH, KIND_HEADER_HEIGHT),
        egui::Sense::hover(),
    );
    let border_top = rect.bottom() - BORDER_HEIGHT;
    let label = single_line_galley(ui, &lint_kind.to_string(), 12.0, FLYOUT_WIDTH - 20.0);

    paint_text(
        ui.painter(),
        rect.left() + 10.0,
        (rect.top() + border_top) / 2.0,
        label,
        TEXT_COLOR,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(egui::pos2(rect.left(), border_top), rect.right_bottom()),
        0.0,
        lint_kind_color32(lint_kind),
    );
}

/// Renders the full-width row that disables the rule that produced the lint.
fn disable_rule_row(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(FLYOUT_WIDTH, DISABLE_ROW_HEIGHT),
        egui::Sense::click(),
    );

    if response.hovered() {
        // This row can touch the card's bottom corners, so keep the hover inside the curves.
        let corner_radius = egui::CornerRadius {
            sw: CARD_CORNER_RADIUS - 1,
            se: CARD_CORNER_RADIUS - 1,
            ..Default::default()
        };
        ui.painter()
            .rect_filled(rect, corner_radius, HOVER_BACKGROUND);
    }

    ui.painter().hline(
        rect.x_range(),
        rect.top() + DIVIDER_WIDTH / 2.0,
        egui::Stroke::new(DIVIDER_WIDTH, DIVIDER_COLOR),
    );

    let color = if response.hovered() {
        MUTED_TEXT_COLOR
    } else {
        SUBTLE_TEXT_COLOR
    };
    let label = single_line_galley(ui, "Disable this rule", 11.0, FLYOUT_WIDTH - 20.0);
    paint_text(
        ui.painter(),
        rect.left() + 10.0,
        (rect.top() + DIVIDER_WIDTH + rect.bottom()) / 2.0,
        label,
        color,
    );

    hover_text(
        response.on_hover_cursor(egui::CursorIcon::PointingHand),
        "Disable the rule that produced this lint",
    )
}

/// Renders a compact button in the style of the Chrome extension's `.harper-btn`.
///
/// The fill darkens on hover, like the CSS `brightness(0.92)` filter.
fn flyout_button(
    ui: &mut egui::Ui,
    content: ButtonContent,
    fill: egui::Color32,
    hover: impl Into<String>,
) -> egui::Response {
    const GLYPH_SIZE: f32 = 18.0;

    let galley = match content {
        ButtonContent::Text(text) => Some(single_line_galley(ui, text, 12.0, f32::INFINITY)),
        ButtonContent::Glyph(_) => None,
    };
    let content_size = galley
        .as_ref()
        .map_or(egui::vec2(GLYPH_SIZE, GLYPH_SIZE), |galley| galley.size());
    let size = egui::vec2(
        content_size.x + 16.0,
        (content_size.y + 6.0).max(BUTTON_MIN_HEIGHT),
    );

    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let fill = if response.hovered() {
        brightness(fill, 0.92)
    } else {
        fill
    };
    ui.painter().rect_filled(rect, 6.0, fill);

    let content_rect = egui::Rect::from_center_size(rect.center(), content_size);
    match (content, galley) {
        (ButtonContent::Glyph(glyph), _) => {
            draw_glyph(ui, content_rect, glyph, BUTTON_TEXT_COLOR);
        }
        (ButtonContent::Text(_), Some(galley)) => {
            ui.painter()
                .galley(content_rect.min, galley, BUTTON_TEXT_COLOR);
        }
        (ButtonContent::Text(_), None) => {}
    }

    hover_text(
        response.on_hover_cursor(egui::CursorIcon::PointingHand),
        hover,
    )
}

/// Renders Harper's markdown-formatted lint message in the popup instead of showing raw markdown
/// markers to the user.
///
/// Inline code gets a background tinted with the lint color. The Chrome extension underlines it
/// instead, but `egui_commonmark` cannot underline inline code.
fn render_lint_message(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    message: &str,
    lint_kind: LintKind,
) {
    ui.scope(|ui| {
        let [r, g, b, _] = lint_kind_color32(lint_kind).to_array();
        let style = ui.style_mut();
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(12.0));
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, egui::FontId::monospace(12.0));
        style.spacing.item_spacing = egui::vec2(0.0, 3.0);
        style.visuals.override_text_color = Some(MUTED_TEXT_COLOR);
        style.visuals.code_bg_color = egui::Color32::from_rgba_unmultiplied(r, g, b, 40);

        CommonMarkViewer::new()
            .default_width(Some(ui.available_width() as usize))
            .show(ui, cache, message);
    });
}

/// Applies tooltip styling that visually belongs with the suggestion popup instead of egui defaults.
fn hover_text(response: egui::Response, hover_text: impl Into<String>) -> egui::Response {
    let hover_text = hover_text.into();

    let mut tooltip = egui::Tooltip::for_enabled(&response);
    tooltip.popup = tooltip
        .popup
        .frame(card_frame().inner_margin(egui::Margin::symmetric(8, 6)));
    tooltip.show(|ui| {
        ui.set_max_width(260.0);
        ui.label(
            egui::RichText::new(hover_text)
                .size(12.0)
                .color(MUTED_TEXT_COLOR),
        );
    });

    response
}

/// Lays out one line of text, cut off with an ellipsis when it is wider than `max_width`.
///
/// The galley uses a placeholder color, so [`paint_text`] can choose the color when it paints.
fn single_line_galley(
    ui: &egui::Ui,
    text: &str,
    size: f32,
    max_width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::single_section(
        text.to_owned(),
        egui::TextFormat::simple(egui::FontId::proportional(size), egui::Color32::PLACEHOLDER),
    );
    job.wrap = egui::text::TextWrapping::truncate_at_width(max_width.max(0.0));

    ui.painter().layout_job(job)
}

/// Paints a galley with its left edge at `left`, centered vertically on `center_y`.
fn paint_text(
    painter: &egui::Painter,
    left: f32,
    center_y: f32,
    galley: std::sync::Arc<egui::Galley>,
    color: egui::Color32,
) {
    let pos = egui::pos2(left, center_y - galley.size().y / 2.0);
    painter.galley(pos, galley, color);
}

/// Draws the small SVG-inspired line icons from the Chrome extension using egui painter primitives.
fn draw_glyph(ui: &egui::Ui, rect: egui::Rect, glyph: Glyph, color: egui::Color32) {
    match glyph {
        Glyph::Sliders => draw_sliders_icon(ui, rect, color),
        Glyph::BookDown => draw_book_down_icon(ui, rect, color),
    }
}

/// Draws Font Awesome's `faSliders`: three horizontal tracks, each with a knob.
fn draw_sliders_icon(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.5_f32, color);
    let knob_radius = rect.height() * 0.15;

    for (track_y, knob_x) in [(0.19, 0.375), (0.5, 0.625), (0.81, 0.3125)] {
        let y = rect.top() + rect.height() * track_y;
        ui.painter().hline(rect.x_range(), y, stroke);
        ui.painter().circle_filled(
            egui::pos2(rect.left() + rect.width() * knob_x, y),
            knob_radius,
            color,
        );
    }
}

/// Draws Lucide's `book-down` icon from its 24×24 SVG path.
fn draw_book_down_icon(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32) {
    const ARC_STEPS: usize = 8;

    let scale = rect.width().min(rect.height()) / 24.0;
    let origin = rect.center() - egui::vec2(12.0, 12.0) * scale;
    let point = |x: f32, y: f32| origin + egui::vec2(x, y) * scale;
    let arc = |center_x: f32, center_y: f32, radius: f32, from_degrees: f32, to_degrees: f32| {
        (0..=ARC_STEPS)
            .map(|step| {
                let t = step as f32 / ARC_STEPS as f32;
                let angle = (from_degrees + (to_degrees - from_degrees) * t).to_radians();
                point(
                    center_x + radius * angle.cos(),
                    center_y + radius * angle.sin(),
                )
            })
            .collect::<Vec<_>>()
    };
    let stroke = egui::Stroke::new(scale.max(1.0), color);

    let mut outline = vec![point(4.0, 19.5)];
    outline.extend(arc(6.5, 4.5, 2.5, 180.0, 270.0));
    outline.extend(arc(19.0, 3.0, 1.0, 270.0, 360.0));
    outline.extend(arc(19.0, 21.0, 1.0, 0.0, 90.0));
    outline.extend(arc(6.5, 19.5, 2.5, 90.0, 270.0));
    outline.push(point(20.0, 17.0));

    ui.painter().add(egui::Shape::line(outline, stroke));
    ui.painter()
        .line_segment([point(12.0, 13.0), point(12.0, 7.0)], stroke);
    ui.painter().add(egui::Shape::line(
        vec![point(9.0, 10.0), point(12.0, 13.0), point(15.0, 10.0)],
        stroke,
    ));
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
        | LintKind::Punctuation
        | LintKind::Readability
        | LintKind::Redundancy
        | LintKind::Regionalism
        | LintKind::Repetition
        | LintKind::Style
        | LintKind::Usage
        | LintKind::WordChoice
        | LintKind::WordOrder => false,
    }
}

/// Converts Harper suggestion variants into row labels. Like the Chrome extension, it falls back to
/// the kind of edit when the replacement text is empty.
fn suggestion_text(suggestion: &Suggestion) -> String {
    match suggestion {
        Suggestion::ReplaceWith(chars) | Suggestion::InsertAfter(chars) if !chars.is_empty() => {
            chars.iter().collect()
        }
        Suggestion::ReplaceWith(_) => "Replace".to_owned(),
        Suggestion::InsertAfter(_) => "Insert After".to_owned(),
        Suggestion::Remove => "Remove".to_owned(),
    }
}

/// Describes the edit that a suggestion row makes, with the same wording as the Chrome extension.
fn suggestion_hover_text(suggestion: &Suggestion, problem_text: &str) -> String {
    match suggestion {
        Suggestion::ReplaceWith(chars) => format!(
            "Click to replace \"{problem_text}\" with \"{}\"",
            chars.iter().collect::<String>()
        ),
        Suggestion::InsertAfter(chars) => format!(
            "Click to insert \"{}\" after \"{problem_text}\"",
            chars.iter().collect::<String>()
        ),
        Suggestion::Remove => format!("Click to remove \"{problem_text}\""),
    }
}

/// Gets the text that the lint flags, for use in popup labels and tooltips.
fn problem_text(lint: &Lint, source_text: &str) -> String {
    let source_chars = source_text.chars().collect::<Vec<_>>();
    lint.get_str(&source_chars)
}

/// Defines popup geometry ahead of rendering so the transparent overlay can enable or disable native
/// hit-testing based on our intended layout, not a previous frame's measured egui output.
///
/// The list card's size comes from its fixed row geometry. The flyout card's content is markdown,
/// so its bounds always have the maximum size. A flyout with less content is drawn smaller, and the
/// overlay catches clicks on the transparent space below it instead of the editor.
///
/// `sidebar_position` is the flyout card's current offset from the list card's top, not its target,
/// so hit-testing follows the card while it moves.
pub fn popup_layout_for_lint(rect: &Rect, lint: &Lint, sidebar_position: f32) -> PopupLayout {
    popup_layout(
        rect,
        UNBOUNDED_VIEWPORT,
        egui::vec2(LIST_CARD_WIDTH, list_card_height(row_count(lint))),
        egui::vec2(FLYOUT_CARD_WIDTH, CARD_HEIGHT),
        sidebar_position,
    )
}

/// Computes the sidebar position that aligns the flyout card with the row at `shown_suggestion`.
///
/// The sidebar moves toward this value on each frame, but does not jump to it.
pub fn sidebar_target(lint: &Lint, shown_suggestion: Option<usize>) -> f32 {
    row_offset(shown_suggestion, list_card_height(row_count(lint)))
}

/// Counts the list card's suggestion rows, including the "No replacements" row.
pub fn row_count(lint: &Lint) -> usize {
    lint.suggestions.len().max(1)
}

/// Computes the list card's height from its fixed row and footer heights.
fn list_card_height(row_count: usize) -> f32 {
    let content = 2.0 * LIST_PADDING_Y + row_count as f32 * ROW_HEIGHT + LIST_FOOTER_HEIGHT;

    (2.0 * BORDER_WIDTH + content).min(CARD_HEIGHT)
}

/// Computes the distance from the list card's top to the top of the row at `shown_suggestion`.
///
/// `None` gives the first row. Like the Chrome extension, this ignores the list's scroll position,
/// but the result never goes past the last row position that fits in the card.
fn row_offset(shown_suggestion: Option<usize>, list_height: f32) -> f32 {
    let offset = FIRST_ROW_OFFSET + shown_suggestion.unwrap_or(0) as f32 * ROW_HEIGHT;

    offset.min(list_height - BORDER_WIDTH - ROW_HEIGHT).max(0.0)
}

/// Scales a color's channels, like the CSS `brightness()` filter.
fn brightness(color: egui::Color32, factor: f32) -> egui::Color32 {
    let [r, g, b, a] = color.to_array();
    let scale = |channel: u8| (f32::from(channel) * factor).round().clamp(0.0, 255.0) as u8;

    egui::Color32::from_rgba_premultiplied(scale(r), scale(g), scale(b), a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_card_height_counts_rows_and_footer() {
        assert_eq!(list_card_height(1), 2.0 + 8.0 + 30.0 + 37.0);
        assert_eq!(list_card_height(3), 2.0 + 8.0 + 90.0 + 37.0);
    }

    #[test]
    fn list_card_height_caps_at_max() {
        assert_eq!(list_card_height(100), CARD_HEIGHT);
    }

    #[test]
    fn row_offset_starts_at_first_row() {
        let list_height = list_card_height(3);

        assert_eq!(row_offset(None, list_height), FIRST_ROW_OFFSET);
        assert_eq!(
            row_offset(Some(0), list_height),
            row_offset(None, list_height)
        );
    }

    #[test]
    fn row_offset_steps_by_row_height() {
        let list_height = list_card_height(3);

        assert_eq!(
            row_offset(Some(2), list_height),
            FIRST_ROW_OFFSET + 2.0 * ROW_HEIGHT
        );
    }

    #[test]
    fn row_offset_stays_inside_scrolled_list() {
        let list_height = list_card_height(100);

        assert_eq!(
            row_offset(Some(99), list_height),
            CARD_HEIGHT - BORDER_WIDTH - ROW_HEIGHT
        );
    }

    #[test]
    fn suggestion_text_uses_replacement() {
        assert_eq!(
            suggestion_text(&Suggestion::ReplaceWith("their".chars().collect())),
            "their"
        );
        assert_eq!(
            suggestion_text(&Suggestion::InsertAfter(",".chars().collect())),
            ","
        );
    }

    #[test]
    fn suggestion_text_falls_back_to_edit_kind() {
        assert_eq!(
            suggestion_text(&Suggestion::ReplaceWith(Vec::new())),
            "Replace"
        );
        assert_eq!(
            suggestion_text(&Suggestion::InsertAfter(Vec::new())),
            "Insert After"
        );
        assert_eq!(suggestion_text(&Suggestion::Remove), "Remove");
    }

    #[test]
    fn suggestion_hover_text_describes_edit() {
        assert_eq!(
            suggestion_hover_text(&Suggestion::ReplaceWith("their".chars().collect()), "there"),
            "Click to replace \"there\" with \"their\""
        );
        assert_eq!(
            suggestion_hover_text(&Suggestion::InsertAfter(",".chars().collect()), "However"),
            "Click to insert \",\" after \"However\""
        );
        assert_eq!(
            suggestion_hover_text(&Suggestion::Remove, "the"),
            "Click to remove \"the\""
        );
    }

    #[test]
    fn brightness_darkens_opaque_colors() {
        assert_eq!(
            brightness(egui::Color32::from_rgb(100, 200, 0), 0.5),
            egui::Color32::from_rgb(50, 100, 0)
        );
    }
}
