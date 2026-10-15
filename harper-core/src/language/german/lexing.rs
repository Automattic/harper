//! German-specific lexing functions.

use crate::lexing::{FoundToken, lex_english_token};

/// Lex a German token from the source.
/// For German, we can reuse the English lexing logic since the tokenization
/// patterns are similar (same character sets, similar word boundaries).
pub(crate) fn lex_german_token(source: &[char]) -> FoundToken {
    // Reuse English lexing for German text
    // This is appropriate because:
    // 1. German uses the same character set as English
    // 2. Word boundaries work the same way
    // 3. Numbers, URLs, emails, etc. are tokenized identically
    // 4. German-specific processing happens at higher levels (spell checking, grammar)
    lex_english_token(source)
}
