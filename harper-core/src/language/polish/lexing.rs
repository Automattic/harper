//! Polish-specific lexing functions.

use crate::lexing::{FoundToken, lex_english_token};

/// Lex a Polish token from the source.
/// For Polish, we can reuse the English lexing logic since the tokenization
/// patterns are similar (same character sets, similar word boundaries).
pub(crate) fn lex_polish_token(source: &[char]) -> FoundToken {
    // Reuse English lexing for Polish text
    // This is appropriate because:
    // 1. Polish uses similar character sets as English
    // 2. Word boundaries work the same way
    // 3. Numbers, URLs, emails, etc. are tokenized identically
    // 4. Polish-specific processing happens at higher levels (spell checking, grammar)
    lex_english_token(source)
}
