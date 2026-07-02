use crate::Token;
use crate::language::slovak::lexing::lex_slovak_token;
use crate::lexing::lex_with;
use crate::parsers::Parser;

/// A parser that will attempt to lex as many tokens as possible,
/// without discrimination and until the end of input.
///
/// Uses Slovak-specific lexing that currently reuses the English lexing
/// logic but is structured to allow future Slovak-specific tokenization
/// if needed.
#[derive(Clone, Copy)]
pub struct PlainSlovak;

impl Parser for PlainSlovak {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        lex_with(source, lex_slovak_token)
    }
}
