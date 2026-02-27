use std::{borrow::Cow, hash::BuildHasher};

use foldhash::fast::FixedState;
use serde::{Deserialize, Serialize};

use crate::{CharString, CharStringExt, spell::CanonicalWordId};

/// An identifier for a particular word with case-folded casing.
///
/// This does not usually point to a specific word, but rather a group of words that are identical
/// when lowercased.
#[derive(Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct CaseFoldedWordId {
    pub(super) hash: u64,
}

impl CaseFoldedWordId {
    /// Create a Word ID from a character slice.
    ///
    /// This will Case-fold and normalize the input before calculating the word ID.
    ///
    /// If the input word was already case-folded, this will also return
    /// a [`CanonicalWordId`]. This is because the IDs will be identical under the hood, and the
    /// canonical ID can be produced with no additional cost.
    pub fn from_word_chars(chars: impl AsRef<[char]>) -> (Self, Option<CanonicalWordId>) {
        let normalized = chars.as_ref().normalized();
        let lower = normalized.to_lower();

        let was_already_case_folded =
            matches!(normalized, Cow::Borrowed(_)) && matches!(lower, Cow::Borrowed(_));

        let hash = FixedState::default().hash_one(lower);

        (
            Self { hash },
            was_already_case_folded.then_some(CanonicalWordId { hash }),
        )
    }

    /// Create a word ID from a string.
    /// Requires allocation, so use sparingly.
    ///
    /// This will case-fold and normalize the input before calculating the word ID.
    ///
    /// If the input word was already case-folded, this will also return
    /// a [`CanonicalWordId`]. This is because the IDs will be identical under the hood, and the
    /// canonical ID can be produced with no additional cost.
    pub fn from_word_str(text: impl AsRef<str>) -> (Self, Option<CanonicalWordId>) {
        let chars: CharString = text.as_ref().chars().collect();
        Self::from_word_chars(chars)
    }
}
