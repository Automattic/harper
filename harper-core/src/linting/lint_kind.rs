use std::fmt::Display;

use is_macro::Is;
use serde::{Deserialize, Serialize};

/// The general category a [`Lint`](super::Lint) falls into.
/// There's no reason not to add a new item here if you are adding a new rule that doesn't fit
/// the existing categories.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Is, Default, Hash, PartialEq, Eq)]
pub enum LintKind {
    /// This should only be used by linters doing spellcheck on individual words.
    Spelling,
    Capitalization,
    Style,
    Formatting,
    Repetition,
    Enhancement,
    Readability,
    WordChoice,
    Punctuation,
    #[default]
    Miscellaneous,
}

impl LintKind {
    pub fn new_from_str(s: &str) -> Option<Self> {
        Some(match s {
            "Spelling" => LintKind::Spelling,
            "Capitalization" => LintKind::Capitalization,
            "Formatting" => LintKind::Formatting,
            "Repetition" => LintKind::Repetition,
            "Readability" => LintKind::Readability,
            "Miscellaneous" => LintKind::Miscellaneous,
            "Enhancement" => LintKind::Enhancement,
            "Word Choice" => LintKind::WordChoice,
            "Style" => LintKind::Style,
            _ => return None,
        })
    }

    /// Produce a string representation, which can be used as keys in a map or CSS variables.
    pub fn to_string_key(&self) -> String {
        match self {
            LintKind::Spelling => "Spelling",
            LintKind::Capitalization => "Capitalization",
            LintKind::Formatting => "Formatting",
            LintKind::Repetition => "Repetition",
            LintKind::Readability => "Readability",
            LintKind::Miscellaneous => "Miscellaneous",
            LintKind::Enhancement => "Enhancement",
            LintKind::WordChoice => "WordChoice",
            LintKind::Style => "Style",
            LintKind::Punctuation => "Punctuation",
        }
        .to_owned()
    }
}

impl Display for LintKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LintKind::Spelling => "Spelling",
            LintKind::Capitalization => "Capitalization",
            LintKind::Formatting => "Formatting",
            LintKind::Repetition => "Repetition",
            LintKind::Readability => "Readability",
            LintKind::Miscellaneous => "Miscellaneous",
            LintKind::Enhancement => "Enhancement",
            LintKind::WordChoice => "Word Choice",
            LintKind::Style => "Style",
            LintKind::Punctuation => "Punctuation",
        };

        write!(f, "{}", s)
    }
}
