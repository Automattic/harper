mod draw_highlight;
mod popup_layout;
mod render_lint_card;

use std::time::Instant;

use egui_commonmark::CommonMarkCache;
use harper_core::{Document, linting::LintKind};

use self::{
    draw_highlight::draw_highlight,
    popup_layout::{PopupLayout, approach},
    render_lint_card::{
        FIRST_ROW_OFFSET, LintCardAction, popup_layout_for_lint, render_lint_card, row_count,
        sidebar_target,
    },
};
use super::{AddToDictionary, DisableRule, IgnoreLint, OpenSettings};
use crate::{
    lint_kind_color::lint_kind_color,
    rect::{ActionableLint, Rect},
};

/// A lint marker, popup, or empty overlay region found during cursor hit-testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTarget {
    Lint(usize),
    Popup,
    None,
}

/// Stores highlighter-specific drawing state and renders it into an egui frame.
///
/// `Window` owns the native window and GPU plumbing; `RenderState` owns the content we want to draw
/// into that plumbing. This keeps future highlight rectangles, styling, and animation state out of
/// the platform/rendering infrastructure.
pub struct RenderState {
    /// Last successful accessibility read, retained when a later read fails.
    last_lints: Option<Vec<ActionableLint>>,

    /// Index of the lint whose suggestion popup is currently visible.
    highlighted_lint: Option<usize>,

    /// Index of the suggestion row that the user hovered last in the visible popup.
    shown_suggestion: Option<usize>,

    /// The flyout card's drawn offset from the list card's top, in points.
    current_sidebar_position: f32,

    /// When `render` last ran, used to move the sidebar at a speed that follows real time.
    last_render: Option<Instant>,

    /// Cache for markdown-rendered lint messages so popup redraws do not repeatedly rebuild markdown
    /// resources from scratch.
    markdown_cache: CommonMarkCache,

    /// Called when the user dismisses a lint so app-level ignored-lint state can be updated.
    ignore_lint: IgnoreLint,

    /// Called when the user adds a spelling lint's source text to the local dictionary.
    add_to_dictionary: AddToDictionary,

    /// Called when the user disables the rule that produced the selected lint.
    disable_rule: DisableRule,

    open_settings: OpenSettings,
}

impl RenderState {
    /// Creates render state through `set_lints` so initial lint data gets the same stale-popup guard
    /// used by later accessibility refreshes.
    pub fn new(
        rects: Vec<ActionableLint>,
        ignore_lint: IgnoreLint,
        add_to_dictionary: AddToDictionary,
        disable_rule: DisableRule,
        open_settings: OpenSettings,
    ) -> Self {
        let mut state = Self {
            last_lints: None,
            highlighted_lint: None,
            shown_suggestion: None,
            current_sidebar_position: FIRST_ROW_OFFSET,
            last_render: None,
            markdown_cache: CommonMarkCache::default(),
            ignore_lint,
            add_to_dictionary,
            disable_rule,
            open_settings,
        };
        state.set_lints(rects);
        state
    }

    /// Saves successful accessibility reads while retaining the previous lints after a failed read.
    ///
    /// Closes the popup when its lint disappeared, and forgets the shown suggestion when the
    /// refreshed lint has fewer rows.
    pub fn set_lints(&mut self, lints: Vec<ActionableLint>) {
        self.last_lints = Some(lints);

        if self
            .highlighted_lint
            .is_some_and(|index| index >= self.lints().len())
        {
            self.close_popup();
        }

        let row_count = self
            .highlighted_lint
            .and_then(|index| self.lints().get(index))
            .map(|positioned_lint| row_count(&positioned_lint.lint));
        self.shown_suggestion = self
            .shown_suggestion
            .filter(|index| row_count.is_some_and(|row_count| *index < row_count));
    }

    fn lints(&self) -> &[ActionableLint] {
        self.last_lints.as_deref().unwrap_or_default()
    }

    /// Updates which lint owns the suggestion popup without exposing render-state internals.
    ///
    /// Window/input code decides what the user interacted with, while `RenderState` owns the popup
    /// selection. Filtering invalid indexes here keeps stale cursor state from selecting a lint that
    /// disappeared after the latest accessibility read.
    ///
    /// Like the Chrome extension, each new selection starts again from the first suggestion row.
    pub fn set_highlighted_lint(&mut self, highlighted_lint: Option<usize>) {
        self.highlighted_lint = highlighted_lint.filter(|index| *index < self.lints().len());
        self.shown_suggestion = None;
        self.current_sidebar_position = FIRST_ROW_OFFSET;
    }

    /// Centralizes the close behavior so the popup close button only has to clear the selected lint.
    pub fn close_popup(&mut self) {
        self.highlighted_lint = None;
        self.shown_suggestion = None;
        self.current_sidebar_position = FIRST_ROW_OFFSET;
    }

    /// Checks if a given position touches a hit target.
    pub fn hit_target_at_pos(&self, pos: egui::Pos2) -> HitTarget {
        if self
            .popup_layout()
            .is_some_and(|layout| layout.contains(pos))
        {
            return HitTarget::Popup;
        }

        self.lints()
            .iter()
            .position(|positioned_lint| {
                rect_bounds(&positioned_lint.rect.with_friendly_padding()).contains(pos)
            })
            .map_or(HitTarget::None, HitTarget::Lint)
    }

    /// Computes popup hit-test bounds from our layout contract instead of waiting for egui to report
    /// rendered bounds, which keeps hit-testing available before the next render pass completes.
    pub fn popup_layout(&self) -> Option<PopupLayout> {
        self.highlighted_lint
            .and_then(|index| self.lints().get(index))
            .map(|positioned_lint| {
                popup_layout_for_lint(
                    &positioned_lint.rect,
                    &positioned_lint.lint,
                    self.current_sidebar_position,
                )
            })
    }

    /// Draws highlights and the active popup from the same state used by hit-testing so visible
    /// regions and clickable regions do not drift apart.
    ///
    /// Each monitor has its own window. `screen_rects` holds the bounds of every window, in the
    /// screen coordinates that lints and hit-testing use, and `window` is the index of the one being
    /// drawn. Lints are moved into that window's coordinates, and only one window draws the popup.
    ///
    /// Each call also moves the sidebar toward its target by the time since the previous call.
    pub fn render(&mut self, ui: &mut egui::Ui, screen_rects: &[egui::Rect], window: usize) {
        let window_rect = screen_rects[window];
        let now = Instant::now();
        let time_since_last_render = self
            .last_render
            .map_or(0.0, |last_render| (now - last_render).as_secs_f32());
        self.last_render = Some(now);

        if let Some(positioned_lint) = self
            .highlighted_lint
            .and_then(|index| self.lints().get(index))
        {
            let target = sidebar_target(&positioned_lint.lint, self.shown_suggestion);
            self.current_sidebar_position = approach(
                self.current_sidebar_position,
                target,
                time_since_last_render,
            );
        }

        for positioned_lint in self.lints() {
            draw_highlight(
                ui,
                &to_window_space(&positioned_lint.rect, window_rect),
                &positioned_lint.lint,
            );
        }

        if let Some(index) = self.highlighted_lint
            && let Some(positioned_lint) = self.lints().get(index)
            && popup_window(screen_rects, &positioned_lint.rect) == Some(window)
        {
            let rect = to_window_space(&positioned_lint.rect, window_rect);
            let lint = positioned_lint.lint.clone();
            let source_text = positioned_lint.source_text.clone();

            match render_lint_card(
                ui,
                &rect,
                &lint,
                &source_text,
                self.shown_suggestion,
                self.current_sidebar_position,
                &mut self.markdown_cache,
            ) {
                Some(LintCardAction::Close) => self.close_popup(),
                Some(LintCardAction::OpenSettings) => {
                    (self.open_settings)();
                    self.close_popup();
                }
                Some(LintCardAction::HoverSuggestion(suggestion)) => {
                    self.shown_suggestion = Some(suggestion);
                }
                Some(LintCardAction::ApplySuggestion(suggestion)) => {
                    if let Some(actionable_lint) = self
                        .last_lints
                        .as_mut()
                        .and_then(|lints| lints.get_mut(index))
                    {
                        actionable_lint.apply_suggestion(suggestion);
                    }

                    self.close_popup();
                }
                Some(LintCardAction::IgnoreLint) => {
                    if let Some((lint, source_text)) =
                        self.lints().get(index).map(|actionable_lint| {
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
                        self.lints().get(index).map(|actionable_lint| {
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
                        .lints()
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

/// Converts accessibility rectangles into egui rectangles so rendering and hit-testing use the same
/// screen-space geometry.
fn rect_bounds(rect: &Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(rect.x as f32, rect.y as f32),
        egui::vec2(rect.width as f32, rect.height as f32),
    )
}

/// Moves a screen-space rectangle into the coordinates of the window at `window_rect`, whose
/// top-left corner egui draws at the origin.
fn to_window_space(rect: &Rect, window_rect: egui::Rect) -> Rect {
    Rect::new(
        rect.x - window_rect.min.x as f64,
        rect.y - window_rect.min.y as f64,
        rect.width,
        rect.height,
    )
}

/// Picks the index of the window that draws the popup for the lint at `lint_rect`: the window that
/// contains the lint's center, or the nearest window when the center is outside all of them.
///
/// Egui moves popups that would be outside a window back inside it, so exactly one window may draw
/// the popup. When the center is on the edge between two windows, the first one is picked.
fn popup_window(screen_rects: &[egui::Rect], lint_rect: &Rect) -> Option<usize> {
    let center = rect_bounds(lint_rect).center();

    screen_rects
        .iter()
        .map(|screen_rect| screen_rect.distance_sq_to_pos(center))
        .enumerate()
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}

fn lint_kind_color32(lint_kind: LintKind) -> egui::Color32 {
    let color = lint_kind_color(lint_kind);

    egui::Color32::from_rgb(color.r, color.g, color.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A built-in display at the origin, with an external display to its left and above it.
    const BUILT_IN: egui::Rect =
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1512.0, 982.0));
    const EXTERNAL: egui::Rect =
        egui::Rect::from_min_max(egui::pos2(-3008.0, -392.0), egui::pos2(0.0, 1300.0));
    const SCREENS: [egui::Rect; 2] = [BUILT_IN, EXTERNAL];

    #[test]
    fn to_window_space_moves_rect_by_window_origin() {
        let rect = Rect::new(-2900.0, -300.0, 30.0, 12.0);

        assert_eq!(
            to_window_space(&rect, EXTERNAL),
            Rect::new(108.0, 92.0, 30.0, 12.0)
        );
        assert_eq!(to_window_space(&rect, BUILT_IN), rect);
    }

    #[test]
    fn popup_window_contains_lint() {
        let on_built_in = Rect::new(100.0, 100.0, 30.0, 12.0);
        let on_external = Rect::new(-2900.0, -300.0, 30.0, 12.0);

        assert_eq!(popup_window(&SCREENS, &on_built_in), Some(0));
        assert_eq!(popup_window(&SCREENS, &on_external), Some(1));
    }

    #[test]
    fn popup_window_is_nearest_when_lint_center_is_off_screen() {
        let below_built_in = Rect::new(100.0, 975.0, 30.0, 20.0);
        let left_of_external = Rect::new(-3030.0, 100.0, 30.0, 12.0);

        assert_eq!(popup_window(&SCREENS, &below_built_in), Some(0));
        assert_eq!(popup_window(&SCREENS, &left_of_external), Some(1));
        assert_eq!(popup_window(&[BUILT_IN], &left_of_external), Some(0));
    }

    #[test]
    fn popup_window_picks_one_window_for_lint_on_shared_edge() {
        let on_edge = Rect::new(-15.0, 100.0, 30.0, 12.0);

        assert_eq!(popup_window(&SCREENS, &on_edge), Some(0));
        assert_eq!(popup_window(&[EXTERNAL, BUILT_IN], &on_edge), Some(0));
    }
}
