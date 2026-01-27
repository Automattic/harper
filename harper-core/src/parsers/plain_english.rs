use super::Parser;
use crate::lexing::{FoundToken, lex_english_token};
use crate::{Span, Token};

/// A parser that will attempt to lex as many tokens as possible,
/// without discrimination and until the end of input.
#[derive(Clone, Copy)]
pub struct PlainEnglish;

impl Parser for PlainEnglish {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        // Lex tokens
        let mut cursor = 0;
        let mut tokens = Vec::new();

        loop {
            if cursor >= source.len() {
                return tokens;
            }

            let FoundToken { token, next_index } = lex_english_token(&source[cursor..]);

            tokens.push(Token {
                span: Span::new(cursor, cursor + next_index),
                kind: token,
            });
            cursor += next_index;
        }
    }
}
