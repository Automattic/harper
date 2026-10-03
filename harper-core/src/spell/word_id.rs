//! Identifiers for words.
//!
//! These are meant for situations where you need to refer to a word (or a collection of words),
//! without storing all of accompanying data (like spelling or metadata).

pub use canonical_word_id::CanonicalWordId;
pub use case_folded_word_id::CaseFoldedWordId;
pub(crate) use word_id_pair::WordIdPair;

mod canonical_word_id;
mod case_folded_word_id;
mod word_id_pair;
