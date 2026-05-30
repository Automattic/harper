//! Plain text parser for Portuguese.

use crate::Token;
use crate::lexing::{lex_portuguese_token, lex_with};
use crate::parsers::Parser;

/// A parser for plain Portuguese text.
#[derive(Clone, Copy, Default)]
pub struct PlainPortuguese;

impl Parser for PlainPortuguese {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        lex_with(source, lex_portuguese_token)
    }
}
