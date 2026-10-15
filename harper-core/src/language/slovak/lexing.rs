//! Slovak-specific lexing functions.

use crate::lexing::{FoundToken, lex_english_token};

/// Lex a Slovak token from the source.
/// For Slovak, we can reuse the English lexing logic since the tokenization
/// patterns are similar (same character sets, similar word boundaries).
pub(crate) fn lex_slovak_token(source: &[char]) -> FoundToken {
    // Reuse English lexing for Slovak text
    // This is appropriate because:
    // 1. Slovak uses the same character set as English (plus diacritics which are handled)
    // 2. Word boundaries work the same way
    // 3. Numbers, URLs, emails, etc. are tokenized identically
    // 4. Slovak-specific processing happens at higher levels (spell checking, grammar)
    lex_english_token(source)
}
