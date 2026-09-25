//! Closed-class grammatical data for German.
//!
//! Determiners and prepositions are finite, irregular and small, so they are
//! written out in Rust instead of being carried by dictionary flags. The
//! dictionary describes one word at a time and gives it a
//! [`crate::language::morphology::Agreement`] whose axes are independent; a
//! determiner's readings are joint, and a preposition's government is a
//! property of the word's *syntax*, not of its form. Neither fits an affix
//! flag.

pub mod determiners;
pub mod prepositions;
