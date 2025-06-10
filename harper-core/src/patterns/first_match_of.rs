use super::Pattern;
use crate::Token;

/// A naive pattern collection that naively iterates through a list of patterns,
/// returning the first one that matches.
///
/// Compare to [`LongestMatchOf`](super::LongestMatchOf), which returns the longest match.
#[derive(Default)]
pub struct FirstMatchOf {
    patterns: Vec<Box<dyn Pattern>>,
}

impl FirstMatchOf {
    pub fn push(&mut self, pattern: Box<dyn Pattern>) {
        self.patterns.push(pattern);
    }
}

impl Pattern for FirstMatchOf {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        self.patterns
            .iter()
            .filter_map(|p| p.matches(tokens, source))
            .next()
    }
}
