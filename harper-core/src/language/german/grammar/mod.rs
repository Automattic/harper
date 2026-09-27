//! Closed-class grammatical data for German.
//!
//! Determiners and prepositions are finite, irregular and small, so they are
//! written out in Rust instead of being carried by dictionary flags. The
//! dictionary describes one word at a time and gives it a
//! [`crate::language::morphology::Agreement`] whose axes are independent; a
//! determiner's readings are joint, and a preposition's government is a
//! property of the word's *syntax*, not of its form. Neither fits an affix
//! flag.
//!
//! [`noun_phrase`] is the odd one out: not a table but a scan, and it is here
//! because three linters need the same answer from it and each used to guess
//! separately.

pub mod determiners;
pub mod noun_phrase;
pub mod prepositions;
pub mod subjects;
