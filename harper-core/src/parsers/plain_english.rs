use super::Parser;
use crate::lexing::{FoundToken, lex_token};
use crate::{Span, Token};

/// A parser that will attempt to lex as many tokens as possible,
/// without discrimination and until the end of input.
#[derive(Clone, Copy)]
pub struct PlainEnglish;

impl Parser for PlainEnglish {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut cursor = 0;

        // Lex tokens
        let mut tokens = Vec::new();

        loop {
            if cursor > source.len() {
                panic!()
            }
            if cursor == source.len() {
                return tokens;
            }

            if let Some(FoundToken { token, next_index }) = lex_token(&source[cursor..]) {
                tokens.push(Token {
                    span: Span::new(cursor, cursor + next_index),
                    kind: token,
                });
                cursor += next_index;
            } else {
                panic!()
            }
        }
    }
}
