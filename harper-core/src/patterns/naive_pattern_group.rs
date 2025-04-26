use std::num::NonZeroUsize;

use super::Pattern;
use crate::Token;

/// A naive pattern collection that naively iterates through a list of patterns,
/// returning the first one that matches.
#[derive(Default)]
pub struct NaivePatternGroup {
    patterns: Vec<Box<dyn Pattern>>,
}

impl NaivePatternGroup {
    pub fn push(&mut self, pattern: Box<dyn Pattern>) {
        self.patterns.push(pattern);
    }
}

impl Pattern for NaivePatternGroup {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        self.patterns
            .iter()
            .filter_map(|p| p.matches(tokens, source))
            .next()
    }
}
