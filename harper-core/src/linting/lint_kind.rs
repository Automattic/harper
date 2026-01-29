use std::fmt::Display;

use is_macro::Is;
use serde::{Deserialize, Serialize};

/// The general category a [`Lint`](super::Lint) falls into.
/// There's no reason not to add a new item here if you are adding a new rule that doesn't fit
/// the existing categories.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Is, Default, Hash, PartialEq, Eq)]
pub enum LintKind {
    Agreement,
    /// For errors where words are joined or split at the wrong boundaries
    /// (e.g., "each and everyone" vs. "each and every one")
    BoundaryError,
    Capitalization,
    /// For cases where a word or phrase is misused for a similar-sounding word or phrase,
    /// where the incorrect version makes logical sense (e.g., 'egg corn' for 'acorn',
    /// 'on mass' for 'en masse').
    Eggcorn,
    /// For suggesting improvements that enhance clarity or impact without fixing errors
    Enhancement,
    Formatting,
    Grammar,
    /// For cases where a word is mistakenly used for a similar-sounding word with a different meaning
    /// (e.g., 'eluded to' instead of 'alluded to'). Unlike eggcorns, these don't create new meanings.
    Malapropism,
    /// For any other lint that doesn't fit neatly into the other categories
    #[default]
    Miscellaneous,
    Nonstandard,
    /// For issues with punctuation, including hyphenation in compound adjectives
    /// (e.g., "face first" vs. "face-first" when used before a noun)
    Punctuation,
    Readability,
    /// For cases where words duplicate meaning that's already expressed
    /// (e.g., "basic fundamentals" → "fundamentals", "free gift" → "gift")
    Redundancy,
    /// For variations that are standard in some regions or dialects but not others
    Regionalism,
    Repetition,
    /// When your brain doesn't know the right spelling.
    /// This should only be used by linters doing spellcheck on individual words.
    Spelling,
    /// For cases where multiple options are correct but one is preferred for style or clarity,
    /// such as expanding abbreviations in formal writing (e.g., 'min' → 'minimum')
    Style,
    /// When your brain knows the right spelling but your fingers made a mistake.
    /// (e.g., 'can be seem' → 'can be seen')
    Typo,
    /// For conventional word usage and standard collocations
    /// (e.g., 'by accident' vs. 'on accident' in standard English)
    Usage,
    /// For choosing between different words or phrases in a given context
    WordChoice,
}

impl LintKind {
    /// Produce a string representation, which can be used as keys in a map or CSS variables.
    pub fn to_string_key(&self) -> String {
        match self {
            LintKind::Agreement => "Agreement",
            LintKind::BoundaryError => "BoundaryError",
            LintKind::Capitalization => "Capitalization",
            LintKind::Eggcorn => "Eggcorn",
            LintKind::Enhancement => "Enhancement",
            LintKind::Formatting => "Formatting",
            LintKind::Grammar => "Grammar",
            LintKind::Malapropism => "Malapropism",
            LintKind::Miscellaneous => "Miscellaneous",
            LintKind::Nonstandard => "Nonstandard",
            LintKind::Punctuation => "Punctuation",
            LintKind::Readability => "Readability",
            LintKind::Redundancy => "Redundancy",
            LintKind::Regionalism => "Regionalism",
            LintKind::Repetition => "Repetition",
            LintKind::Spelling => "Spelling",
            LintKind::Style => "Style",
            LintKind::Typo => "Typo",
            LintKind::Usage => "Usage",
            LintKind::WordChoice => "WordChoice",
        }
        .to_owned()
    }
}

impl Display for LintKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LintKind::Agreement => "Agreement",
            LintKind::BoundaryError => "BoundaryError",
            LintKind::Capitalization => "Capitalization",
            LintKind::Eggcorn => "Eggcorn",
            LintKind::Enhancement => "Enhancement",
            LintKind::Formatting => "Formatting",
            LintKind::Grammar => "Grammar",
            LintKind::Malapropism => "Malapropism",
            LintKind::Miscellaneous => "Miscellaneous",
            LintKind::Nonstandard => "Nonstandard",
            LintKind::Punctuation => "Punctuation",
            LintKind::Readability => "Readability",
            LintKind::Redundancy => "Redundancy",
            LintKind::Regionalism => "Regionalism",
            LintKind::Repetition => "Repetition",
            LintKind::Spelling => "Spelling",
            LintKind::Style => "Style",
            LintKind::Typo => "Typo",
            LintKind::Usage => "Usage",
            LintKind::WordChoice => "Word Choice",
        };

        write!(f, "{s}")
    }
}
