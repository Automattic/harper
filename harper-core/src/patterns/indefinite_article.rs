use crate::Token;

use super::{SingleTokenPattern, WordSet};

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

impl SingleTokenPattern for IndefiniteArticle {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        self.inner.matches_token(token, source)
    }
}
