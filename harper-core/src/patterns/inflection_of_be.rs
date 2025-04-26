use super::Pattern;
use crate::Token;
use crate::patterns::WordSet;

/// Matches any inflection of the verb “be”:
/// `am`, `is`, `are`, `was`, `were`, `be`, `been`, `being`.
pub struct InflectionOfBe {
    inner: WordSet,
}

impl InflectionOfBe {
    /// Create a new `BeInflection` matcher.
    pub fn new() -> Self {
        Self {
            inner: WordSet::new(&["be", "am", "is", "are", "was", "were", "been", "being"]),
        }
    }
}

impl Pattern for InflectionOfBe {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        self.inner.matches(tokens, source)
    }
}
