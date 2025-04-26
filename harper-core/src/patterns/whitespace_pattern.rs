use std::num::NonZeroUsize;

use super::Pattern;

pub struct WhitespacePattern;

impl Pattern for WhitespacePattern {
    fn matches(&self, tokens: &[crate::Token], _source: &[char]) -> Option<NonZeroUsize> {
        NonZeroUsize::new(
            tokens
                .iter()
                .position(|t| !t.kind.is_whitespace())
                .unwrap_or(tokens.len()),
        )
    }
}
