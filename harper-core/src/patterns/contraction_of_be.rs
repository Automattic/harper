use super::SingleTokenPattern;
use crate::Token;
use crate::patterns::WordSet;

/// Matches any contraction of the verb “be”:
/// `I'm`, `we're`, `you're`, `he's`, `she's`, `it's`, `they're`.
pub struct ContractionOfBe {
    /// If using a `WordSet` proves expensive, we'll switch to something else.
    inner: WordSet,
}

impl Default for ContractionOfBe {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractionOfBe {
    pub fn new() -> Self {
        Self {
            inner: WordSet::new(&["i'm", "we're", "you're", "he's", "she's", "it's", "they're"]),
        }
    }
    // TODO: include_common_errors() ctor à la `ReflexivePronoun``
}

impl SingleTokenPattern for ContractionOfBe {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        self.inner.matches_token(token, source)
    }
}
