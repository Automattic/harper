use crate::{LSend, Token, spell::Dictionary};

use super::SingleTokenPattern;

/// A single token pattern that accepts a closure with access to a dictionary for custom matching logic.
pub struct DictionaryToken<F, D>
where
    F: LSend + Fn(&Token, &[char], &D) -> bool,
    D: Dictionary + ?Sized,
{
    matcher: F,
    dict: D,
}

impl<F, D> DictionaryToken<F, D>
where
    F: LSend + Fn(&Token, &[char], &D) -> bool,
    D: Dictionary + ?Sized,
{
    /// Creates a new token pattern with the provided matching closure and dictionary.
    pub fn new(matcher: F, dict: D) -> Self
    where
        D: Sized,
    {
        Self { matcher, dict }
    }
}

impl<F, D> SingleTokenPattern for DictionaryToken<F, D>
where
    F: LSend + Fn(&Token, &[char], &D) -> bool,
    D: Dictionary + ?Sized,
{
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        (self.matcher)(token, source, &self.dict)
    }
}
