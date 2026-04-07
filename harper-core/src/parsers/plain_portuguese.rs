use super::Parser;
use crate::Token;
use crate::lexing::{lex_portuguese_token, lex_with};

/// A parser that will attempt to lex as many tokens as possible,
/// without discrimination and until the end of input.
#[derive(Clone, Copy)]
pub struct PlainPortuguese;

impl Parser for PlainPortuguese {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        lex_with(source, lex_portuguese_token)
    }
}
