use super::SingleTokenPattern;
use crate::Token;
use crate::patterns::WordSet;

/// Matches any inflection of the verb "need":
/// `need`, `needs`, `needed`, `needing`.
pub struct InflectionOfNeed {
    /// If using a `WordSet` proves expensive, we'll switch to something else.
    inner: WordSet,
}

impl Default for InflectionOfNeed {
    fn default() -> Self {
        Self::new()
    }
}

impl InflectionOfNeed {
    pub fn new() -> Self {
        Self {
            inner: WordSet::new(&["need", "needs", "needed", "needing"]),
        }
    }
}

impl SingleTokenPattern for InflectionOfNeed {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        self.inner.matches_token(token, source)
    }
}
