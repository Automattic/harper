use std::num::NonZeroUsize;

use crate::{Dictionary, Token, TokenStringExt, make_title_case};

use super::Pattern;

/// Will match full length of wrapped pattern only if the matched
/// text is not already title case.
pub struct IsNotTitleCase<D: Dictionary> {
    inner: Box<dyn Pattern>,
    dict: D,
}

impl<D: Dictionary> IsNotTitleCase<D> {
    pub fn new(inner: Box<dyn Pattern>, dict: D) -> Self {
        Self { inner, dict }
    }
}

impl<D: Dictionary> Pattern for IsNotTitleCase<D> {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        let inner_match = self.inner.matches(tokens, source)?;

        let matched_chars = tokens[0..inner_match.get()]
            .span()
            .unwrap()
            .get_content(source);
        if make_title_case(&tokens[0..inner_match.get()], source, &self.dict) != matched_chars {
            Some(inner_match)
        } else {
            None
        }
    }
}
