use super::Parser;
use crate::Token;
use crate::lexing::{lex_english_token, lex_with};

/// A parser that will attempt to lex as many tokens as possible,
/// without discrimination and until the end of input.
#[derive(Clone, Copy)]
pub struct PlainEnglish;

impl Parser for PlainEnglish {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        lex_with(source, lex_english_token)
    }
}
