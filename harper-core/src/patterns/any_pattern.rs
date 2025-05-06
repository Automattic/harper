use crate::Token;

use super::Pattern;

/// A [`Pattern`] that will match any single token.
pub struct AnyPattern;

impl Pattern for AnyPattern {
    fn matches(&self, tokens: &[Token], _source: &[char]) -> Option<usize> {
        if tokens.is_empty() { None } else { Some(1) }
    }
}
