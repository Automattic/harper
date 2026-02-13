use std::hash::BuildHasher;

use foldhash::fast::FixedState;
use serde::{Deserialize, Serialize};

use super::CaseFoldedWordId;
use crate::CharString;

/// An identifier for a particular word with canonical casing.
#[derive(Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct CanonicalWordId {
    pub(super) hash: u64,
}

impl CanonicalWordId {
    /// Create a Word ID from a character slice.
    pub fn from_word_chars(chars: impl AsRef<[char]>) -> Self {
        let hash = FixedState::default().hash_one(chars.as_ref());

        Self { hash }
    }

    /// Create a word ID from a string.
    /// Requires allocation, so use sparingly.
    pub fn from_word_str(text: impl AsRef<str>) -> Self {
        let chars: CharString = text.as_ref().chars().collect();
        Self::from_word_chars(chars)
    }

    /// Reinterpret this ID as a [`CaseFoldedWordId`].
    ///
    /// Note that this is just a reinterpretation, it does not perform any conversion. This is
    /// useful when the canonical word ID is the same as the case-folded word ID. This will only
    /// happen if the canonical ID was generated with a word that was already lowercased and
    /// normalized.
    pub(crate) fn as_case_folded(self) -> CaseFoldedWordId {
        CaseFoldedWordId { hash: self.hash }
    }
}
