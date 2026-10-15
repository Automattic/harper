//! Portuguese-specific lexing functions.

use crate::lexing::email_address::lex_email_address;
use crate::lexing::hostname::lex_hostname_token;
use crate::lexing::url::lex_url;
use crate::lexing::{
    FoundToken, lex_catch, lex_hex_number, lex_newlines, lex_number, lex_punctuation, lex_regexish,
    lex_spaces, lex_tabs, lex_word,
};

/// Lex a Portuguese token from the source.
pub(crate) fn lex_portuguese_token(source: &[char]) -> FoundToken {
    [
        lex_regexish,
        lex_punctuation,
        lex_tabs,
        lex_spaces,
        lex_newlines,
        // lex_plural_digit, // The Portuguese language doesn't have this feature
        lex_hex_number, // Before lex_number, which would match the initial 0
        // lex_long_decade,  // This works in other ways in Portuguese
        lex_number,
        lex_url,
        lex_email_address,
        lex_hostname_token,
        lex_word,
    ]
    .into_iter()
    .find_map(|lexer| lexer(source))
    .unwrap_or_else(lex_catch)
}
