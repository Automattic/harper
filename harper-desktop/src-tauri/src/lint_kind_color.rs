use harper_core::linting::LintKind;

use crate::color::Color;

pub const LINT_KINDS: [LintKind; 21] = [
    LintKind::Agreement,
    LintKind::BoundaryError,
    LintKind::Capitalization,
    LintKind::Eggcorn,
    LintKind::Enhancement,
    LintKind::Formatting,
    LintKind::Grammar,
    LintKind::Malapropism,
    LintKind::Miscellaneous,
    LintKind::Nonstandard,
    LintKind::Preposition,
    LintKind::Punctuation,
    LintKind::Readability,
    LintKind::Redundancy,
    LintKind::Regionalism,
    LintKind::Repetition,
    LintKind::Spelling,
    LintKind::Style,
    LintKind::Typo,
    LintKind::Usage,
    LintKind::WordChoice,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextColor {
    Black,
    White,
}

pub fn lint_kind_color(lint_kind: LintKind) -> Color {
    match lint_kind {
        LintKind::Agreement => Color::new(0x22, 0x8B, 0x22),
        LintKind::BoundaryError => Color::new(0x8B, 0x45, 0x13),
        LintKind::Capitalization => Color::new(0x54, 0x0D, 0x6E),
        LintKind::Eggcorn => Color::new(0xFF, 0x8C, 0x00),
        LintKind::Enhancement => Color::new(0x0E, 0xAD, 0x69),
        LintKind::Formatting => Color::new(0x7D, 0x3C, 0x98),
        LintKind::Grammar => Color::new(0x9B, 0x59, 0xB6),
        LintKind::Malapropism => Color::new(0xC7, 0x15, 0x85),
        LintKind::Miscellaneous => Color::new(0x3B, 0xCE, 0xAC),
        LintKind::Nonstandard => Color::new(0x00, 0x8B, 0x8B),
        LintKind::Preposition => Color::new(0xFF, 0x7F, 0x50),
        LintKind::Punctuation => Color::new(0xD4, 0x85, 0x0F),
        LintKind::Readability => Color::new(0x2E, 0x8B, 0x57),
        LintKind::Redundancy => Color::new(0x46, 0x82, 0xB4),
        LintKind::Regionalism => Color::new(0xC0, 0x61, 0xCB),
        LintKind::Repetition => Color::new(0x00, 0xA6, 0x7C),
        LintKind::Spelling => Color::new(0xEE, 0x42, 0x66),
        LintKind::Style => Color::new(0xFF, 0xD2, 0x3F),
        LintKind::Typo => Color::new(0xFF, 0x6B, 0x35),
        LintKind::Usage => Color::new(0x1E, 0x90, 0xFF),
        LintKind::WordChoice => Color::new(0x22, 0x8B, 0x22),
        LintKind::WordOrder => Color::new(0x4D, 0x4D, 0xFF),
    }
}

pub fn lint_kind_color_from_key(lint_kind_key: &str) -> Option<Color> {
    LintKind::from_string_key(lint_kind_key).map(lint_kind_color)
}

pub fn lint_kind_text_color(lint_kind: LintKind) -> TextColor {
    contrasting_text_color(lint_kind_color(lint_kind))
}

pub fn contrasting_text_color(color: Color) -> TextColor {
    let luminance =
        0.299 * f64::from(color.r) + 0.587 * f64::from(color.g) + 0.114 * f64::from(color.b);

    if luminance > 186.0 {
        TextColor::Black
    } else {
        TextColor::White
    }
}
