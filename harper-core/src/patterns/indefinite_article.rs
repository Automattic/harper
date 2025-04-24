use std::num::NonZeroUsize;

use crate::Token;

use super::{Pattern, SequencePattern, WordSet};

pub struct IndefiniteArticle {
    inner: SequencePattern,
}

impl Default for IndefiniteArticle {
    fn default() -> Self {
        Self {
            inner: SequencePattern::default().then(WordSet::new(&["a", "an"])),
        }
    }
}

impl Pattern for IndefiniteArticle {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        self.inner.matches(tokens, source)
    }
}
