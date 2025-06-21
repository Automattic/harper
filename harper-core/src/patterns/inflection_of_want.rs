use super::SingleTokenPattern;
use crate::Token;
use crate::patterns::WordSet;

/// Matches any inflection of the verb "want":
/// `want`, `wants`, `wanted`, `wanting`.
pub struct InflectionOfWant {
    /// If using a `WordSet` proves expensive, we'll switch to something else.
    inner: WordSet,
}

impl Default for InflectionOfWant {
    fn default() -> Self {
        Self::new()
    }
}

impl InflectionOfWant {
    pub fn new() -> Self {
        Self {
            inner: WordSet::new(&["want", "wants", "wanted", "wanting"]),
        }
    }
}

impl SingleTokenPattern for InflectionOfWant {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        self.inner.matches_token(token, source)
    }
}
