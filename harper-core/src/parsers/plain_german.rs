use super::Parser;
use crate::Token;
use crate::lexing::{lex_english_token, lex_with};

/// A parser that will attempt to lex as many tokens as possible,
/// without discrimination and until the end of input.
///
/// This is the German language parser, which handles German-specific
/// tokenization including compound words and special characters.
#[derive(Clone, Copy)]
pub struct PlainGerman;

impl Parser for PlainGerman {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        // For now, we use the same lexing as English
        // TODO: Implement German-specific tokenization:
        // - German quotation marks („" instead of ")
        // - Special characters (ä, ö, ü, ß)
        // - Compound word boundaries
        lex_with(source, lex_english_token)
    }
}