use crate::Token;
use crate::language::polish::lexing::lex_polish_token;
use crate::lexing::lex_with;
use crate::parsers::Parser;

/// A parser that will attempt to lex as many tokens as possible,
/// without discrimination and until the end of input.
///
/// Uses Polish-specific lexing that currently reuses the English lexing
/// logic but is structured to allow future Polish-specific tokenization
/// if needed.
#[derive(Clone, Copy)]
pub struct PlainPolish;

impl Parser for PlainPolish {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        lex_with(source, lex_polish_token)
    }
}
