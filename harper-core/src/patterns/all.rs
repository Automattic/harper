use std::num::NonZeroUsize;

use crate::Token;

use super::Pattern;

/// A [`Pattern`] that consumes a list of other patterns and only
/// matches if all the child patterns do.
///
/// It will match the length of the longest pattern.
#[derive(Default)]
pub struct All {
    children: Vec<Box<dyn Pattern>>,
}

impl All {
    pub fn new(children: Vec<Box<dyn Pattern>>) -> Self {
        Self { children }
    }

    pub fn add(&mut self, p: Box<dyn Pattern>) {
        self.children.push(p);
    }
}

impl Pattern for All {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        let mut max = 0;

        for pattern in &self.children {
            let len = pattern.matches(tokens, source)?;
            max = max.max(len.get());
        }

        NonZeroUsize::new(max)
    }
}
