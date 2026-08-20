use crate::Token;
use crate::language::german::lexing::lex_german_token;
use crate::lexing::lex_with;
use crate::parsers::Parser;

/// A parser that will attempt to lex as many tokens as possible,
/// without discrimination and until the end of input.
///
/// Uses German-specific lexing that currently reuses the English lexing
/// logic but is structured to allow future German-specific tokenization
/// if needed.
#[derive(Clone, Copy)]
pub struct PlainGerman;

impl Parser for PlainGerman {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        lex_with(source, lex_german_token)
    }
}
