//! Identifiers for words.
//!
//! These are meant for situations where you need to refer to a word (or a collection of words),
//! without storing all of accompanying data (like spelling or metadata).

use serde::{Deserialize, Serialize};

use crate::CharString;

pub use canonical_word_id::CanonicalWordId;
pub use case_folded_word_id::CaseFoldedWordId;

mod canonical_word_id;
mod case_folded_word_id;

/// A pair containing both [`CanonicalWordId`] and [`CaseFoldedWordId`] for a given word.
#[derive(Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub(crate) struct WordIdPair {
    canonical: CanonicalWordId,
    case_folded: CaseFoldedWordId,
}

impl WordIdPair {
    /// Create a Word ID pair from a character slice.
    ///
    /// Calculates both the canonical and case-folded word ID for the provided word.
    pub(crate) fn from_word_chars(chars: impl AsRef<[char]>) -> Self {
        let (case_folded, maybe_canonical) = CaseFoldedWordId::from_word_chars(chars.as_ref());

        Self {
            // Trying to avoid hashing again if possible.
            canonical: maybe_canonical.unwrap_or_else(|| CanonicalWordId::from_word_chars(chars)),
            case_folded,
        }
    }

    /// Create a word ID pair from a string.
    /// Requires allocation, so use sparingly.
    ///
    /// Calculates both the canonical and case-folded word ID for the provided word.
    pub(crate) fn from_word_str(text: impl AsRef<str>) -> Self {
        let chars: CharString = text.as_ref().chars().collect();
        Self::from_word_chars(chars)
    }

    /// The canonical ID of the word.
    pub(crate) fn canonical(&self) -> CanonicalWordId {
        self.canonical
    }

    /// The case-folded ID of the word.
    pub(crate) fn case_folded(&self) -> CaseFoldedWordId {
        self.case_folded
    }
}
