use std::sync::Arc;

use crate::{CharString, Dictionary, FstDictionary, Token, WordMetadata};

use super::{Pattern, SequencePattern};

/// A [`Pattern`] that identifies adjacent words that could potentially be merged into a single word.
///
/// This checks if two adjacent words could form a valid compound word, but first verifies
/// that the two words aren't already a valid lexeme in the dictionary (like "straight away").
pub struct MergeableWords {
    inner: SequencePattern,
    dict: Arc<FstDictionary>,
    predicate: Box<dyn Fn(&WordMetadata) -> bool + Send + Sync>,
}

impl MergeableWords {
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

    /// Get the merged word from the dictionary if these words can be merged.
    /// Returns None if the words should remain separate (either because they form
    /// a valid open compound, or don't form a valid closed compound when merged).
    pub fn get_merged_word(
        &self,
        word_a: &Token,
        word_b: &Token,
        source: &[char],
    ) -> Option<CharString> {
        let a_chars: CharString = word_a.span.get_content(source).into();
        let b_chars: CharString = word_b.span.get_content(source).into();

        // First check if the two words with a space exist in the dictionary
        let mut two_word_lexeme = a_chars.clone();
        two_word_lexeme.push(' ');
        two_word_lexeme.extend_from_slice(&b_chars);

        if self.dict.get_word_metadata(&two_word_lexeme).is_some() {
            return None; // Valid two-word lexeme found, don't suggest merging
        }

        // Only suggest merging if the two-word lexeme doesn't exist
        let mut merged = a_chars;
        merged.extend_from_slice(&b_chars);

        if let Some(metadata) = self.dict.get_word_metadata(&merged) {
            if (self.predicate)(&metadata) {
                let correct = self.dict.get_correct_capitalization_of(&merged).unwrap();
                merged.clear();
                merged.extend_from_slice(correct);
                return Some(merged);
            }
        }

        None
    }
}

impl Pattern for MergeableWords {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        let inner_match = self.inner.matches(tokens, source)?;

        if inner_match != 3 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;

    #[test]
    fn merges_compound_not_in_dict() {
        // "note book" is not in the dictionary
        let doc = Document::new_plain_english_curated("note book");
        let a = doc.tokens().nth(0).unwrap();
        let b = doc.tokens().nth(2).unwrap();

        let mergeable = MergeableWords::new(|meta| {
            let result = meta.is_noun();
            result
        });

        let merged = mergeable.get_merged_word(&a, &b, doc.get_source());

        assert_eq!(merged, Some("notebook".chars().collect()));
    }

    #[test]
    fn does_not_merge_open_compound_in_dict() {
        // "straight away" is in the dictionary as an open compound
        let doc = Document::new_plain_english_curated("straight away");
        let a = doc.tokens().nth(0).unwrap();
        let b = doc.tokens().nth(2).unwrap();

        let mergeable = MergeableWords::new(|meta| meta.is_noun());
        let merged = mergeable.get_merged_word(&a, &b, doc.get_source());

        assert_eq!(merged, None);
    }

    #[test]
    fn does_not_merge_invalid_compound() {
        // "quickfox" is not a valid word in the dictionary
        let doc = Document::new_plain_english_curated("quick fox");
        let a = doc.tokens().nth(0).unwrap();
        let b = doc.tokens().nth(2).unwrap();

        let mergeable = MergeableWords::new(|meta| meta.is_noun());
        let merged = mergeable.get_merged_word(&a, &b, doc.get_source());

        assert_eq!(merged, None);
    }
}
