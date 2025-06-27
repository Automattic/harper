use std::fmt::Display;

use is_macro::Is;
use serde::{Deserialize, Serialize};

/// The general category a [`Lint`](super::Lint) falls into.
/// There's no reason not to add a new item here if you are adding a new rule that doesn't fit
/// the existing categories.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Is, Default, Hash, PartialEq, Eq)]
pub enum LintKind {
    Capitalization,
    Enhancement,
    Formatting,
    #[default]
    Miscellaneous,
    Punctuation,
    Readability,
    Redundancy,
    Regionalism,
    Repetition,
    /// This should only be used by linters doing spellcheck on individual words.
    Spelling,
    Style,
    WordChoice,
}

impl LintKind {
    pub fn new_from_str(s: &str) -> Option<Self> {
        Some(match s {
            "Capitalization" => LintKind::Capitalization,
            "Enhancement" => LintKind::Enhancement,
            "Formatting" => LintKind::Formatting,
            "Miscellaneous" => LintKind::Miscellaneous,
            "Readability" => LintKind::Readability,
            "Redundancy" => LintKind::Redundancy,
            "Regionalism" => LintKind::Regionalism,
            "Repetition" => LintKind::Repetition,
            "Spelling" => LintKind::Spelling,
            "Style" => LintKind::Style,
            "Word Choice" => LintKind::WordChoice,
            _ => return None,
        })
    }

    /// Produce a string representation, which can be used as keys in a map or CSS variables.
    pub fn to_string_key(&self) -> String {
        match self {
            LintKind::Capitalization => "Capitalization",
            LintKind::Enhancement => "Enhancement",
            LintKind::Formatting => "Formatting",
            LintKind::Miscellaneous => "Miscellaneous",
            LintKind::Punctuation => "Punctuation",
            LintKind::Readability => "Readability",
            LintKind::Redundancy => "Redundancy",
            LintKind::Regionalism => "Regionalism",
            LintKind::Repetition => "Repetition",
            LintKind::Spelling => "Spelling",
            LintKind::Style => "Style",
            LintKind::WordChoice => "WordChoice",
        }
        .to_owned()
    }
}

impl Display for LintKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LintKind::Capitalization => "Capitalization",
            LintKind::Enhancement => "Enhancement",
            LintKind::Formatting => "Formatting",
            LintKind::Miscellaneous => "Miscellaneous",
            LintKind::Punctuation => "Punctuation",
            LintKind::Readability => "Readability",
            LintKind::Redundancy => "Redundancy",
            LintKind::Regionalism => "Regionalism",
            LintKind::Repetition => "Repetition",
            LintKind::Spelling => "Spelling",
            LintKind::Style => "Style",
            LintKind::WordChoice => "Word Choice",
        };

        write!(f, "{s}")
    }
}
