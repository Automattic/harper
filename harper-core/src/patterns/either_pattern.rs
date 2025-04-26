use std::num::NonZeroUsize;

use crate::Token;

use super::Pattern;

/// A pattern that returns the value of the longest match in a list.
#[derive(Default)]
pub struct EitherPattern {
    patterns: Vec<Box<dyn Pattern>>,
}

impl EitherPattern {
    pub fn new(patterns: Vec<Box<dyn Pattern>>) -> Self {
        Self { patterns }
    }

    pub fn add(&mut self, pattern: Box<dyn Pattern>) {
        self.patterns.push(pattern);
    }
}

impl Pattern for EitherPattern {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        let mut longest = 0;

        for pattern in self.patterns.iter() {
            let match_len = pattern.matches(tokens, source).map_or(0, NonZeroUsize::get);

            if match_len > longest {
                longest = match_len
            }
        }

        NonZeroUsize::new(longest)
    }
}
