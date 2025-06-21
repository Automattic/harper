use super::SingleTokenPattern;
use crate::Token;
use crate::patterns::WordSet;

/// Matches any inflection of the verb "have":
/// `have`, `has`, `had`, `having`.
pub struct InflectionOfHave {
    /// If using a `WordSet` proves expensive, we'll switch to something else.
    inner: WordSet,
}

impl Default for InflectionOfHave {
    fn default() -> Self {
        Self::new()
    }
}

impl InflectionOfHave {
    pub fn new() -> Self {
        Self {
            inner: WordSet::new(&["have", "has", "had", "having"]),
        }
    }
}

impl SingleTokenPattern for InflectionOfHave {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        self.inner.matches_token(token, source)
    }
}
