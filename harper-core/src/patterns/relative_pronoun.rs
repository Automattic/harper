use crate::Token;

use super::{SingleTokenPattern, WordSet};

pub struct RelativePronoun {
    inner: WordSet,
}

impl Default for RelativePronoun {
    fn default() -> Self {
        Self {
            inner: WordSet::new(&["that", "which", "who", "whom", "whose"]),
        }
    }
}

impl SingleTokenPattern for RelativePronoun {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        self.inner.matches_token(token, source)
    }
}
