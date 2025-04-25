use std::num::NonZeroUsize;

use crate::{CharString, Token};

use super::Pattern;

/// A [`Pattern`] that matches any capitalization of a provided word.
#[derive(Clone)]
pub struct AnyCapitalization {
    word: CharString,
}

impl AnyCapitalization {
    pub fn new(word: CharString) -> Self {
        Self { word }
    }

    pub fn of(word: &str) -> Self {
        let chars = word.chars().collect();

        Self::new(chars)
    }
}

impl Pattern for AnyCapitalization {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        let tok = tokens.first()?;

        if !tok.kind.is_word() {
            return None;
        }

        if tok.span.len() != self.word.len() {
            return None;
        }

        let tok_chars = tok.span.get_content(source);

        let partial_match = tok_chars
            .iter()
            .zip(&self.word)
            .all(|(a, b)| a.eq_ignore_ascii_case(b));

        NonZeroUsize::new(if partial_match { 1 } else { 0 })
    }
}
