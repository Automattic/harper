use crate::color::Color;
use harper_core::linting::{Lint, Suggestion};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColoredRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Color,
}

/// A Harper lint paired with the screen-space rectangle where it should be rendered.
///
/// Harper owns the lint details and accessibility owns the geometry; this type keeps both pieces
/// together without duplicating lint fields into highlighter-specific strings.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedLint {
    pub rect: Rect,
    pub lint: Lint,
}

/// A Harper lint paired with geometry, source context, and an OS-specific suggestion action.
///
/// The highlighter owns rendering and interaction, but the OS broker is the only layer that knows how
/// to mutate the underlying text element. Storing a one-shot callback here keeps those responsibilities
/// connected without teaching rendering code about Accessibility APIs.
pub struct ActionableLint {
    pub rect: Rect,
    pub rule_name: String,
    pub lint: Lint,
    pub source_text: String,
    apply_suggestion: Option<Box<dyn FnOnce(Suggestion)>>,
}

impl PositionedLint {
    pub fn new(rect: Rect, lint: Lint) -> Self {
        Self { rect, lint }
    }
}

impl ActionableLint {
    pub fn new(
        rect: Rect,
        rule_name: String,
        lint: Lint,
        source_text: String,
        apply_suggestion: impl FnOnce(Suggestion) + 'static,
    ) -> Self {
        Self {
            rect,
            rule_name,
            lint,
            source_text,
            apply_suggestion: Some(Box::new(apply_suggestion)),
        }
    }

    pub fn apply_suggestion(&mut self, suggestion: Suggestion) {
        if let Some(apply_suggestion) = self.apply_suggestion.take() {
            apply_suggestion(suggestion);
        }
    }
}

impl ColoredRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64, color: Color) -> Self {
        Self {
            x,
            y,
            width,
            height,
            color,
        }
    }
}
