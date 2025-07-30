use std::sync::Arc;

use crate::{CharString, Dictionary, FstDictionary, Token, WordMetadata};

use super::{LongestMatchOf, Pattern, SequencePattern, WhitespacePattern};

type PredicateFn = dyn Fn(Option<&WordMetadata>, Option<&WordMetadata>) -> bool + Send + Sync;

/// A [`Pattern`] that identifies adjacent words that could potentially be merged into a single word.
///
/// This checks if two adjacent words could form a valid compound word, but first verifies
/// that the two words aren't already a valid lexeme in the dictionary (like "straight away").
pub struct MergeableWords {
    inner: SequencePattern,
    dict: Arc<FstDictionary>,
    predicate: Box<PredicateFn>,
}

impl MergeableWords {
    pub fn new(
        predicate: impl Fn(Option<&WordMetadata>, Option<&WordMetadata>) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner: SequencePattern::default()
                .then_any_word()
                .then(LongestMatchOf::new(vec![
                    Box::new(WhitespacePattern),
                    Box::new(|tok: &Token, _source: &[char]| tok.kind.is_hyphen()),
                ]))
                .then_any_word(),
            dict: FstDictionary::curated(),
            predicate: Box::new(predicate),
        }
    }

    /// Get the merged word from the dictionary if these words can be merged.
    /// Returns None if the words should remain separate (according to the predicate).
    pub fn get_merged_word(
        &self,
        word_a: &Token,
        word_b: &Token,
        source: &[char],
    ) -> Option<CharString> {
        let a_chars: CharString = word_a.span.get_content(source).into();
        let b_chars: CharString = word_b.span.get_content(source).into();

        // First check if the open compound exists in the dictionary
        let mut compound = a_chars.clone();
        compound.push(' ');
        compound.extend_from_slice(&b_chars);
        let meta_open = self.dict.get_word_metadata(&compound);

        // Then check if the closed compound exists in the dictionary
        compound.remove(a_chars.len());
        let meta_closed = self.dict.get_word_metadata(&compound);

        if (self.predicate)(meta_closed, meta_open) {
            return Some(compound);
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

        if self
            .get_merged_word(&tokens[0], &tokens[2], source)
            .is_some()
        {
            return Some(inner_match);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{Document, WordMetadata, patterns::MergeableWords};

    fn predicate(meta_closed: Option<&WordMetadata>, meta_open: Option<&WordMetadata>) -> bool {
        meta_open.is_none() && meta_closed.map_or(false, |m| m.is_noun() && !m.is_proper_noun())
    }

    #[test]
    fn merges_open_compound_not_in_dict() {
        // note book is not in the dictionary, but notebook is
        let doc = Document::new_plain_english_curated("note book");
        let a = doc.tokens().nth(0).unwrap();
        let b = doc.tokens().nth(2).unwrap();

        let merged = MergeableWords::new(predicate).get_merged_word(&a, &b, doc.get_source());

        assert_eq!(merged, Some("notebook".chars().collect()));
    }

    #[test]
    fn does_not_merge_open_compound_in_dict() {
        // straight away is in the dictionary, and straightaway is
        let doc = Document::new_plain_english_curated("straight away");
        let a = doc.tokens().nth(0).unwrap();
        let b = doc.tokens().nth(2).unwrap();

        let merged = MergeableWords::new(predicate).get_merged_word(&a, &b, doc.get_source());

        assert_eq!(merged, None);
    }

    #[test]
    fn does_not_merge_invalid_compound() {
        // neither quick for nor quickfox are in the dictionary
        let doc = Document::new_plain_english_curated("quick fox");
        let a = doc.tokens().nth(0).unwrap();
        let b = doc.tokens().nth(2).unwrap();

        let merged = MergeableWords::new(predicate).get_merged_word(&a, &b, doc.get_source());

        assert_eq!(merged, None);
    }

    #[test]
    fn merges_open_compound() {
        // Dictionary has "frontline" but not "front line"
        let doc = Document::new_plain_english_curated("front line");
        let a = doc.tokens().nth(0).unwrap();
        let b = doc.tokens().nth(2).unwrap();

        let merged = MergeableWords::new(predicate).get_merged_word(&a, &b, doc.get_source());

        assert_eq!(merged, Some("frontline".chars().collect()));
    }

    #[test]
    fn merges_hyphenated_compound() {
        // Doesn't check for "front-line" in the dictionary but matches it and "frontline" is in the dictionary
        let doc = Document::new_plain_english_curated("front-line");
        let a = doc.tokens().nth(0).unwrap();
        let b = doc.tokens().nth(2).unwrap();

        let merged = MergeableWords::new(predicate).get_merged_word(&a, &b, doc.get_source());

        assert_eq!(merged, Some("frontline".chars().collect()));
    }
}
