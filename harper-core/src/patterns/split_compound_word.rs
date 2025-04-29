use std::{num::NonZeroUsize, sync::Arc};

use crate::{CharString, Dictionary, FstDictionary, Token, WordMetadata};

use super::{Pattern, SequencePattern};

/// A [`Pattern`] that looks for valid words accidentally split by whitespace.
///
/// Note that matches of this pattern are not guaranteed to be valid if merged.
pub struct SplitCompoundWord {
    inner: SequencePattern,
    dict: Arc<FstDictionary>,
    predicate: Box<dyn Fn(&WordMetadata) -> bool + Send + Sync>,
}

impl SplitCompoundWord {
    /// Create a new instance of the linter which will only look for compound words that fit the
    /// provided predicate.
    pub fn new(predicate: impl Fn(&WordMetadata) -> bool + Send + Sync + 'static) -> Self {
        Self {
            inner: SequencePattern::default()
                .then_any_word()
                .then_whitespace()
                .then_any_word(),
            dict: FstDictionary::curated(),
            predicate: Box::new(predicate),
        }
    }

    /// Get the merged word from the dictionary that this pattern would match on if it was split.
    pub fn get_merged_word(
        &self,
        word_a: &Token,
        word_b: &Token,
        source: &[char],
    ) -> Option<CharString> {
        let a_chars: CharString = word_a.span.get_content(source).into();
        let b_chars: CharString = word_b.span.get_content(source).into();

        let mut buffer = CharString::new();

        buffer.clear();
        buffer.extend_from_slice(&a_chars);
        buffer.extend_from_slice(&b_chars);

        if let Some(metadata) = self.dict.get_word_metadata(&buffer) {
            if (self.predicate)(metadata) {
                let correct = self.dict.get_correct_capitalization_of(&buffer).unwrap();
                buffer.clear();
                buffer.extend_from_slice(correct);
                return Some(buffer);
            }
        }

        None
    }
}

impl Pattern for SplitCompoundWord {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        let inner_match = self.inner.matches(tokens, source)?;

        if inner_match.get() != 3 {
            return None;
        }

        let a = &tokens[0];
        let b = &tokens[2];

        if self.get_merged_word(a, b, source).is_some() {
            return Some(inner_match);
        }

        None
    }
}
