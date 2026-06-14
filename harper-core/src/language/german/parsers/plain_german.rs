use crate::Token;
use crate::lexing::{lex_english_token, lex_with};
use crate::parsers::Parser;

/// A parser that will attempt to lex as many tokens as possible,
/// without discrimination and until the end of input.
///
/// Today this shares the same lexer as [`super::PlainEnglish`]. That is
/// sufficient for the current German LSP path and keeps parser selection
/// dialect-aware without perturbing English behavior. If Harper adds more
/// language-specific tokenization later, this is the type to extend.
#[derive(Clone, Copy)]
pub struct PlainGerman;

impl Parser for PlainGerman {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        lex_with(source, lex_english_token)
    }
}
