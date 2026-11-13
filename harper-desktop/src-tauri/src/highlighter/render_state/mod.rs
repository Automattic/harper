mod draw_highlight;
mod popup_layout;
mod render_lint_card;
mod tips;

use std::time::Instant;

use egui_commonmark::CommonMarkCache;
use harper_core::{Document, linting::LintKind};

use self::{
    draw_highlight::draw_highlight,
    popup_layout::{PopupLayout, approach},
    render_lint_card::{
        FIRST_ROW_OFFSET, LintCardAction, TipLayout, popup_layout_for_lint, render_lint_card,
        row_count, sidebar_target,
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

    /// The tip chosen when this popup opened. `None` is a retained decision not to show a tip,
    /// not a request to reroll on the next frame; `highlighted_lint` identifies the session.
    current_tip: Option<&'static str>,

    /// Text measured with the active egui fonts before drawing. Hit-testing reuses its height
    /// between frames. A newly selected tip has no extra hit area until its first render.
    tip_layout: Option<TipLayout>,

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
            current_tip: None,
            tip_layout: None,
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
    /// Preserves the tip through geometry-only refreshes, but closes the popup if its index no
    /// longer refers to the same rule, span, and source text. Forget the shown suggestion when the
    /// refreshed lint has fewer rows. Conservatively closing on source changes prevents a popup
    /// (and its tip) from silently moving to another occurrence after an edit or lint reordering.
    ///
    /// Returns whether the display needs repainting. Compare data rather than callbacks: even an
    /// identical display must receive fresh suggestion actions targeting the latest OS element.
    pub fn set_lints(&mut self, lints: Vec<ActionableLint>) -> bool {
        let changed = !same_lint_display(self.lints(), &lints);
        let popup_before = (self.highlighted_lint, self.shown_suggestion);
        let same_lint = self.highlighted_lint.is_some_and(|index| {
            self.lints()
                .get(index)
                .zip(lints.get(index))
                .is_some_and(|(old, new)| {
                    old.rule_name == new.rule_name
                        && old.lint.span == new.lint.span
                        && old.source_text == new.source_text
                })
        });
        if !same_lint {
            self.close_popup();
        }
        self.last_lints = Some(lints);

        let row_count = self
            .highlighted_lint
            .and_then(|index| self.lints().get(index))
            .map(|positioned_lint| row_count(&positioned_lint.lint));
        self.shown_suggestion = self
            .shown_suggestion
            .filter(|index| row_count.is_some_and(|row_count| *index < row_count));

        changed || popup_before != (self.highlighted_lint, self.shown_suggestion)
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
    /// Like the Chrome extension, each new selection starts again from the first suggestion row
    /// and makes one tip decision. Clicking the already-open lint does not reroll its tip.
    pub fn set_highlighted_lint(&mut self, highlighted_lint: Option<usize>) {
        self.open_popup(highlighted_lint, || {
            tips::choose_tip(&mut fastrand::Rng::new())
        });
    }

    /// Applies a popup transition, invoking the supplied selector only for a new valid session.
    fn open_popup(
        &mut self,
        highlighted_lint: Option<usize>,
        choose_tip: impl FnOnce() -> Option<&'static str>,
    ) {
        let highlighted_lint = highlighted_lint.filter(|index| *index < self.lints().len());
        if highlighted_lint.is_none() {
            self.close_popup();
            return;
        }
        if self.highlighted_lint != highlighted_lint {
            self.current_tip = choose_tip();
            self.tip_layout = None;
        }
        self.highlighted_lint = highlighted_lint;
        self.shown_suggestion = None;
        self.current_sidebar_position = FIRST_ROW_OFFSET;
        self.last_render = None;
    }

    /// Clears all popup-session state, including a selected tip, for every close path.
    pub fn close_popup(&mut self) {
        self.highlighted_lint = None;
        self.shown_suggestion = None;
        self.current_tip = None;
        self.tip_layout = None;
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

    /// Computes hit-test bounds from fixed row geometry and the tip's most recent text measurement.
    /// `render` updates that measurement before computing and drawing the same bounds; until a new
    /// tip's first render, only the suggestion rows and footer have a hit area.
    pub fn popup_layout(&self) -> Option<PopupLayout> {
        self.highlighted_lint
            .and_then(|index| self.lints().get(index))
            .map(|positioned_lint| {
                popup_layout_for_lint(
                    &positioned_lint.rect,
                    &positioned_lint.lint,
                    self.current_sidebar_position,
                    self.tip_layout.as_ref().map_or(0.0, TipLayout::height),
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
    /// Only the popup's owning window advances the sidebar animation and requests further frames.
    /// Once it reaches its target, an unchanged popup needs no ongoing repaint.
    pub fn render(&mut self, ui: &mut egui::Ui, screen_rects: &[egui::Rect], window: usize) {
        let window_rect = screen_rects[window];
        // egui caches text layout internally. Measuring each frame also handles font/DPI changes.
        self.tip_layout = self
            .current_tip
            .map(|tip| TipLayout::new(ui.painter(), tip));
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
            let target = sidebar_target(&positioned_lint.lint, self.shown_suggestion);
            let now = Instant::now();
            let dt = self
                .last_render
                .map_or(0.0, |last| (now - last).as_secs_f32());
            self.last_render = Some(now);
            self.current_sidebar_position = approach(self.current_sidebar_position, target, dt);

            let positioned_lint = &self.lints()[index];
            let rect = to_window_space(&positioned_lint.rect, window_rect);
            let layout = popup_layout_for_lint(
                &rect,
                &positioned_lint.lint,
                self.current_sidebar_position,
                self.tip_layout.as_ref().map_or(0.0, TipLayout::height),
            );
            let lint = positioned_lint.lint.clone();
            let source_text = positioned_lint.source_text.clone();
            let popup_before = (self.highlighted_lint, self.shown_suggestion);

            match render_lint_card(
                ui,
                layout,
                &lint,
                &source_text,
                self.shown_suggestion,
                self.tip_layout.as_ref(),
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

            // Actions happen after painting. Draw their result (including a cleared popup) on the
            // next frame, and keep the custom flyout animation alive only until it reaches target.
            let animating = self.highlighted_lint.is_some_and(|index| {
                self.current_sidebar_position
                    != sidebar_target(&self.lints()[index].lint, self.shown_suggestion)
            });
            if animating || popup_before != (self.highlighted_lint, self.shown_suggestion) {
                ui.ctx().request_repaint();
            }
        }
    }
}

/// Compares the ordered, display-relevant lint data without comparing OS suggestion callbacks.
/// Source text and rule identity also affect popup validity even when rectangles have not moved.
fn same_lint_display(old: &[ActionableLint], new: &[ActionableLint]) -> bool {
    old.len() == new.len()
        && old.iter().zip(new).all(|(old, new)| {
            old.rect == new.rect
                && old.lint == new.lint
                && old.rule_name == new.rule_name
                && old.source_text == new.source_text
        })
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
    use harper_core::{
        Span,
        linting::{Lint, Suggestion},
    };
    use std::{cell::Cell, rc::Rc, time::Duration};

    fn lint_at(x: f64) -> ActionableLint {
        ActionableLint::new(
            Rect::new(x, 100.0, 40.0, 20.0),
            "ExampleRule".to_owned(),
            Lint {
                span: Span::new(0, 3),
                message: "Example message".to_owned(),
                suggestions: vec![
                    Suggestion::Remove,
                    Suggestion::ReplaceWith("the".chars().collect()),
                ],
                ..Lint::default()
            },
            "teh word".to_owned(),
            |_| {},
        )
    }

    fn state(lints: Vec<ActionableLint>) -> RenderState {
        RenderState::new(
            lints,
            Box::new(|_, _| {}),
            Box::new(|_| {}),
            Box::new(|_| {}),
            Box::new(|| {}),
        )
    }

    fn paint_frame(
        state: &mut RenderState,
        context: &egui::Context,
        window: usize,
        events: Vec<egui::Event>,
    ) -> Duration {
        // Advance the custom animation deterministically without sleeping in tests.
        state.last_render = Some(Instant::now() - Duration::from_millis(16));
        let output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    SCREENS[window].size(),
                )),
                events,
                ..Default::default()
            },
            |ui| state.render(ui, &SCREENS, window),
        );
        output.viewport_output[&egui::ViewportId::ROOT].repaint_delay
    }

    #[test]
    fn unchanged_empty_lints_do_not_repaint() {
        let mut state = state(Vec::new());
        assert!(!state.set_lints(Vec::new()));
    }

    #[test]
    fn identical_lints_do_not_repaint_or_close_popup() {
        let mut state = state(vec![lint_at(100.0)]);
        state.open_popup(Some(0), || Some("Keep this tip"));
        assert!(!state.set_lints(vec![lint_at(100.0)]));
        assert_eq!(state.highlighted_lint, Some(0));
        assert_eq!(state.current_tip, Some("Keep this tip"));
    }

    #[test]
    fn changed_lint_data_repaints() {
        let changes: [fn(&mut ActionableLint); 11] = [
            |lint| lint.rect.x += 1.0,
            |lint| lint.rect.y += 1.0,
            |lint| lint.rect.width += 1.0,
            |lint| lint.rect.height += 1.0,
            |lint| lint.rule_name.push('!'),
            |lint| lint.source_text.push('!'),
            |lint| lint.lint.message.push('!'),
            |lint| lint.lint.span = Span::new(4, 8),
            |lint| lint.lint.lint_kind = LintKind::Spelling,
            |lint| lint.lint.suggestions.clear(),
            |lint| lint.lint.priority = 1,
        ];
        for change in changes {
            let mut state = state(vec![lint_at(100.0)]);
            let mut changed = lint_at(100.0);
            change(&mut changed);
            assert!(state.set_lints(vec![changed]));
        }
    }

    #[test]
    fn adding_removing_and_reordering_lints_repaints() {
        let mut state = state(Vec::new());
        assert!(state.set_lints(vec![lint_at(100.0)]));
        assert!(state.set_lints(vec![lint_at(100.0), lint_at(200.0)]));
        assert!(state.set_lints(vec![lint_at(200.0), lint_at(100.0)]));
        assert!(state.set_lints(vec![lint_at(100.0)]));
        assert!(state.set_lints(Vec::new()));
        assert!(!state.set_lints(Vec::new()));
    }

    #[test]
    fn unchanged_display_still_replaces_suggestion_callback() {
        let applied = Rc::new(Cell::new(false));
        let mut state = state(vec![lint_at(100.0)]);
        let lint = lint_at(100.0);
        let callback_applied = applied.clone();
        let updated = ActionableLint::new(
            lint.rect,
            lint.rule_name,
            lint.lint,
            lint.source_text,
            move |_| callback_applied.set(true),
        );
        assert!(!state.set_lints(vec![updated]));
        state.last_lints.as_mut().unwrap()[0].apply_suggestion(Suggestion::Remove);
        assert!(applied.get());
    }

    #[test]
    fn moving_lint_preserves_popup_but_repaints() {
        let mut state = state(vec![lint_at(100.0)]);
        state.open_popup(Some(0), || None);
        assert!(state.set_lints(vec![lint_at(200.0)]));
        assert_eq!(state.highlighted_lint, Some(0));
    }

    #[test]
    fn changed_source_closes_popup_and_repaints() {
        let mut state = state(vec![lint_at(100.0)]);
        state.open_popup(Some(0), || None);
        let mut changed = lint_at(100.0);
        changed.source_text.push('!');
        assert!(state.set_lints(vec![changed]));
        assert_eq!(state.highlighted_lint, None);
    }

    #[test]
    fn removed_lint_closes_popup_and_repaints() {
        let mut state = state(vec![lint_at(100.0)]);
        state.open_popup(Some(0), || None);
        assert!(state.set_lints(Vec::new()));
        assert_eq!(state.highlighted_lint, None);
    }

    #[test]
    fn removed_suggestion_clears_selected_row_and_repaints() {
        let mut state = state(vec![lint_at(100.0)]);
        state.open_popup(Some(0), || None);
        state.shown_suggestion = Some(1);
        let mut changed = lint_at(100.0);
        changed.lint.suggestions.pop();
        assert!(state.set_lints(vec![changed]));
        assert_eq!(state.highlighted_lint, Some(0));
        assert_eq!(state.shown_suggestion, None);
    }

    #[test]
    fn static_overlay_stops_requesting_frames() {
        let context = egui::Context::default();
        let mut state = state(vec![lint_at(100.0)]);
        for _ in 0..5 {
            paint_frame(&mut state, &context, 0, Vec::new());
        }
        assert_eq!(
            paint_frame(&mut state, &context, 0, Vec::new()),
            Duration::MAX
        );
    }

    #[test]
    fn sidebar_animation_requests_frames_then_stops() {
        let context = egui::Context::default();
        let mut state = state(vec![lint_at(100.0)]);
        state.open_popup(Some(0), || None);
        for _ in 0..30 {
            paint_frame(&mut state, &context, 0, Vec::new());
        }
        state.shown_suggestion = Some(1);
        assert_eq!(
            paint_frame(&mut state, &context, 0, Vec::new()),
            Duration::ZERO
        );
        for _ in 0..100 {
            paint_frame(&mut state, &context, 0, Vec::new());
        }
        assert_eq!(
            state.current_sidebar_position,
            sidebar_target(&state.lints()[0].lint, Some(1))
        );
        assert_eq!(
            paint_frame(&mut state, &context, 0, Vec::new()),
            Duration::MAX
        );
    }

    #[test]
    fn popup_animation_does_not_repaint_other_monitors() {
        let context = egui::Context::default();
        let mut state = state(vec![lint_at(100.0)]);
        state.open_popup(Some(0), || None);
        state.shown_suggestion = Some(1);
        for _ in 0..5 {
            paint_frame(&mut state, &context, 1, Vec::new());
        }
        assert_eq!(state.current_sidebar_position, FIRST_ROW_OFFSET);
        assert_eq!(
            paint_frame(&mut state, &context, 1, Vec::new()),
            Duration::MAX
        );
    }

    #[test]
    fn applying_suggestion_requests_frame_to_clear_popup() {
        let context = egui::Context::default();
        let mut state = state(vec![lint_at(100.0)]);
        state.open_popup(Some(0), || None);
        for _ in 0..30 {
            paint_frame(&mut state, &context, 0, Vec::new());
        }
        let pos = state.popup_layout().unwrap().list.min + egui::vec2(80.0, 20.0);
        for pressed in [true, false] {
            let delay = paint_frame(
                &mut state,
                &context,
                0,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            assert_eq!(delay, Duration::ZERO);
        }
        assert_eq!(state.highlighted_lint, None);
        for _ in 0..30 {
            paint_frame(&mut state, &context, 0, Vec::new());
        }
        assert_eq!(
            paint_frame(&mut state, &context, 0, Vec::new()),
            Duration::MAX
        );
    }

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
    fn popup_with_tip_uses_window_coordinates_and_preserves_hit_test_size() {
        let rect = Rect::new(-2900.0, -300.0, 30.0, 12.0);
        let lint = harper_core::linting::Lint::default();

        for tip_height in [0.0, 80.0] {
            let screen_layout = popup_layout_for_lint(&rect, &lint, FIRST_ROW_OFFSET, tip_height);
            let window_layout = popup_layout_for_lint(
                &to_window_space(&rect, EXTERNAL),
                &lint,
                FIRST_ROW_OFFSET,
                tip_height,
            );

            assert_eq!(
                window_layout.list,
                screen_layout.list.translate(-EXTERNAL.min.to_vec2())
            );
            assert_eq!(
                window_layout.flyout,
                screen_layout.flyout.translate(-EXTERNAL.min.to_vec2())
            );
        }
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
