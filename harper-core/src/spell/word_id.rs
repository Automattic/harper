//! Identifiers for a words.
//!
//! These are meant for situations where you need to refer to a word (or a collection of words),
//! without storing all of accompanying data (like spelling or metadata).

use std::hash::BuildHasher;

use foldhash::fast::FixedState;
use serde::{Deserialize, Serialize};

use crate::{CharString, CharStringExt};

/// An identifier for a particular word with canonical casing.
#[derive(Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct CanonicalWordId {
    hash: u64,
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
}

/// An identifier for a particular word with case-folded casing.
///
/// This does not usually point to a specific word, but rather a group of words that are identical
/// when lowercased.
#[derive(Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct CaseFoldedWordId {
    hash: u64,
}

impl CaseFoldedWordId {
    /// Create a Word ID from a character slice.
    ///
    /// This will case-fold and normalize the input before calculating the word ID.
    pub fn from_word_chars(chars: impl AsRef<[char]>) -> Self {
        let normalized = chars.as_ref().normalized();
        let lower = normalized.to_lower();
        let hash = FixedState::default().hash_one(lower);

        Self { hash }
    }

    /// Create a word ID from a string.
    /// Requires allocation, so use sparingly.
    ///
    /// This will case-fold and normalize the input before calculating the word ID.
    pub fn from_word_str(text: impl AsRef<str>) -> Self {
        let chars: CharString = text.as_ref().chars().collect();
        Self::from_word_chars(chars)
    }
}

/// A pair containing both [`CanonicalWordId`] and [`CaseFoldedWordId`] for a given word.
#[derive(Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct WordIdPair {
    canonical: CanonicalWordId,
    case_folded: CaseFoldedWordId,
}

impl WordIdPair {
    /// Create a Word ID pair from a character slice.
    ///
    /// Calculates both the canonical and case-folded word ID for the provided word.
    pub fn from_word_chars(chars: impl AsRef<[char]>) -> Self {
        Self {
            canonical: CanonicalWordId::from_word_chars(&chars),
            case_folded: CaseFoldedWordId::from_word_chars(&chars),
        }
    }

    /// Create a word ID pair from a string.
    /// Requires allocation, so use sparingly.
    ///
    /// Calculates both the canonical and case-folded word ID for the provided word.
    pub fn from_word_str(text: impl AsRef<str>) -> Self {
        let chars: CharString = text.as_ref().chars().collect();
        Self::from_word_chars(chars)
    }

    /// The canonical ID of the word.
    pub fn canonical(&self) -> CanonicalWordId {
        self.canonical
    }

    /// The case-folded ID of the word.
    pub fn case_folded(&self) -> CaseFoldedWordId {
        self.case_folded
    }
}

/// Represents either a canonical or case-folded word ID.
#[derive(Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum EitherWordId {
    Canonical(CanonicalWordId),
    CaseFolded(CaseFoldedWordId),
}
impl EitherWordId {
    /// Create a canonical Word ID from a character slice.
    pub fn from_chars_canonical(chars: impl AsRef<[char]>) -> Self {
        Self::Canonical(CanonicalWordId::from_word_chars(chars))
    }

    /// Create a canonical word ID from a string.
    /// Requires allocation, so use sparingly.
    pub fn from_str_canonical(text: impl AsRef<str>) -> Self {
        Self::Canonical(CanonicalWordId::from_word_str(text))
    }

    /// Create a case-folded Word ID from a character slice.
    ///
    /// This will case-fold and normalize the input before calculating the word ID.
    pub fn from_chars_case_folded(chars: impl AsRef<[char]>) -> Self {
        Self::CaseFolded(CaseFoldedWordId::from_word_chars(chars))
    }

    /// Create a case-folded word ID from a string.
    /// Requires allocation, so use sparingly.
    ///
    /// This will case-fold and normalize the input before calculating the word ID.
    pub fn from_str_case_folded(text: impl AsRef<str>) -> Self {
        Self::CaseFolded(CaseFoldedWordId::from_word_str(text))
    }
}
