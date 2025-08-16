use crate::Token;

use super::SingleTokenPattern;

/// Matches any single token.
pub struct AnyPattern;

impl SingleTokenPattern for AnyPattern {
    fn matches_token(&self, _token: &Token, _source: &[char]) -> bool {
        true
    }
}
