use crate::Token;

use super::{Pattern, WordSet};

pub struct IndefiniteArticle {
    inner: WordSet,
}

impl Default for IndefiniteArticle {
    fn default() -> Self {
        Self {
            inner: WordSet::new(&["a", "an"]),
        }
    }
}

impl Pattern for IndefiniteArticle {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        self.inner.matches(tokens, source)
    }
}
