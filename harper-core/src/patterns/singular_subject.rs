use crate::Token;

use super::{EitherPattern, OwnedPatternExt, Pattern, SequencePattern};

pub struct SingularSubject {
    inner: EitherPattern,
}

impl Default for SingularSubject {
    fn default() -> Self {
        Self {
            inner: SequencePattern::default()
                .then_pronoun()
                .or(Box::new(SequencePattern::default().then_proper_nominal())),
        }
    }
}

impl Pattern for SingularSubject {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        self.inner.matches(tokens, source)
    }
}
