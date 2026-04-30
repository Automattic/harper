use crate::color::Color;
use harper_core::linting::Lint;

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

impl PositionedLint {
    pub fn new(rect: Rect, lint: Lint) -> Self {
        Self { rect, lint }
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
