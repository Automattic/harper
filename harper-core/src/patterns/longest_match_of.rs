use crate::Token;

use super::Pattern;

/// A pattern that returns the value of the longest match in a list.
#[derive(Default)]
pub struct LongestMatchOf {
    patterns: Vec<Box<dyn Pattern>>,
}

impl LongestMatchOf {
    pub fn new(patterns: Vec<Box<dyn Pattern>>) -> Self {
        Self { patterns }
    }

    pub fn add(&mut self, pattern: Box<dyn Pattern>) {
        self.patterns.push(pattern);
    }
}

impl Pattern for LongestMatchOf {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        let mut longest = None;

        for pattern in self.patterns.iter() {
            let Some(match_len) = pattern.matches(tokens, source) else {
                continue;
            };

            if let Some(longest_len) = longest {
                if match_len > longest_len {
                    longest = Some(match_len);
                }
            } else {
                longest = Some(match_len);
            }
        }

        longest
    }
}
