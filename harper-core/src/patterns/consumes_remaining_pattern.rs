use std::num::NonZeroUsize;

use super::Pattern;
use crate::Token;

/// A pattern that wraps another pattern.
/// If the wrapped pattern matches the remainder of the input, it returns the
/// input's length. Otherwise, it matches nothing.
pub struct ConsumesRemainingPattern {
    inner: Box<dyn Pattern>,
}

impl ConsumesRemainingPattern {
    pub fn new(inner: Box<dyn Pattern>) -> Self {
        Self { inner }
    }
}

impl Pattern for ConsumesRemainingPattern {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        self.inner
            .matches(tokens, source)
            .filter(|match_len| match_len.get() == tokens.len())
    }
}
