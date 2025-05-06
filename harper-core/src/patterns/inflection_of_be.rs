use super::Pattern;
use crate::Token;
use crate::patterns::WordSet;

/// Matches any inflection of the verb “be”:
/// `am`, `is`, `are`, `was`, `were`, `be`, `been`, `being`.
pub struct InflectionOfBe {
    /// If using a `WordSet` proves expensive, we'll switch to something else.
    inner: WordSet,
}

impl Default for InflectionOfBe {
    fn default() -> Self {
        Self::new()
    }
}

impl InflectionOfBe {
    pub fn new() -> Self {
        Self {
            inner: WordSet::new(&["be", "am", "is", "are", "was", "were", "been", "being"]),
        }
    }
}

impl Pattern for InflectionOfBe {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        self.inner.matches(tokens, source)
    }
}
