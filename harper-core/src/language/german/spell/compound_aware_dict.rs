//! Compound-aware dictionary wrapper for German.
//!
//! This module provides a dictionary wrapper that adds lazy compound word checking
//! to an existing base dictionary, avoiding the memory explosion of pre-generating
//! all possible compound combinations.

use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use crate::DictWordMetadata;
use crate::spell::{Dictionary, FstDictionary};

use super::compound_checker::CompoundChecker;

/// A compound-aware dictionary that wraps a base dictionary and adds lazy compound checking.
#[derive(Clone)]
///
/// This dictionary first checks the base dictionary, and if a word is not found there,
/// it uses the compound checker to determine if the word can be decomposed into valid
/// compound parts.
pub struct CompoundAwareDictionary {
    /// The base dictionary to check first (FST for fast lookups)
    base_dict: Arc<FstDictionary>,
    /// The compound checker for lazy compound decomposition
    compound_checker: Arc<Mutex<CompoundChecker>>,
}

impl CompoundAwareDictionary {
    /// Create a new CompoundAwareDictionary from a base dictionary and word list
    pub fn new(base_dict: Arc<FstDictionary>, compound_checker: CompoundChecker) -> Self {
        Self {
            base_dict,
            compound_checker: Arc::new(Mutex::new(compound_checker)),
        }
    }

    /// Get a reference to the base dictionary
    pub fn base_dict(&self) -> &Arc<FstDictionary> {
        &self.base_dict
    }

    /// Create a new CompoundAwareDictionary from a base dictionary and word list
    pub fn from_word_list(
        base_dict: Arc<FstDictionary>,
        word_list: &[crate::spell::rune::word_list::AnnotatedWord],
    ) -> Self {
        let compound_checker = CompoundChecker::new(word_list);
        Self::new(base_dict, compound_checker)
    }

    /// Check if the word is in the base dictionary first
    fn is_in_base_dict(&self, word: &[char]) -> bool {
        self.base_dict.contains_word(word)
    }

    /// Check if the word is a compound word
    fn is_compound_word(&self, word: &[char]) -> bool {
        self.compound_checker.lock().unwrap().is_compound_word(word)
    }

    /// Get compound metadata for a word
    fn get_compound_metadata(&self, word: &[char]) -> Option<DictWordMetadata> {
        self.compound_checker
            .lock()
            .unwrap()
            .get_compound_metadata(word)
    }
}

impl Dictionary for CompoundAwareDictionary {
    /// Check if the dictionary contains any capitalization of a given word.
    fn contains_word(&self, word: &[char]) -> bool {
        // Check base dictionary first
        if self.base_dict.contains_word(word) {
            return true;
        }

        // Fall back to compound checking
        self.is_compound_word(word)
    }

    /// Check if the dictionary contains any capitalization of a given word.
    fn contains_word_str(&self, word: &str) -> bool {
        self.contains_word(&word.chars().collect::<Vec<_>>())
    }

    /// Check if the dictionary contains the exact capitalization of a given word.
    fn contains_exact_word(&self, word: &[char]) -> bool {
        // Check base dictionary first
        if self.base_dict.contains_exact_word(word) {
            return true;
        }

        // Fall back to compound checking
        self.is_compound_word(word)
    }

    /// Check if the dictionary contains the exact capitalization of a given word.
    fn contains_exact_word_str(&self, word: &str) -> bool {
        self.contains_exact_word(&word.chars().collect::<Vec<_>>())
    }

    /// Gets best fuzzy match from dictionary
    fn fuzzy_match(
        &'_ self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<crate::spell::FuzzyMatchResult<'_>> {
        // For now, delegate to base dictionary for fuzzy matching
        // Compound words are checked exactly, not fuzzily
        self.base_dict.fuzzy_match(word, max_distance, max_results)
    }

    /// Gets best fuzzy match from dictionary
    fn fuzzy_match_str(
        &'_ self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<crate::spell::FuzzyMatchResult<'_>> {
        self.base_dict
            .fuzzy_match_str(word, max_distance, max_results)
    }

    /// Get the correct capitalization of a word
    fn get_correct_capitalization_of(&self, word: &[char]) -> Option<&'_ [char]> {
        self.base_dict.get_correct_capitalization_of(word)
    }

    /// Get the associated metadata for any capitalization of a given word.
    fn get_word_metadata(&self, word: &[char]) -> Option<Cow<'_, DictWordMetadata>> {
        // Check base dictionary first
        if let Some(metadata) = self.base_dict.get_word_metadata(word) {
            return Some(metadata);
        }

        // If it's a compound word, return compound metadata
        if self.is_compound_word(word) {
            let compound_metadata = self.get_compound_metadata(word)?;
            return Some(Cow::Owned(compound_metadata));
        }

        None
    }

    /// Get the associated metadata for any capitalization of a given word.
    fn get_word_metadata_str(&self, word: &str) -> Option<Cow<'_, DictWordMetadata>> {
        self.get_word_metadata(&word.chars().collect::<Vec<_>>())
    }

    /// Iterate over the words in the dictionary.
    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        // For iteration, we can only return the base dictionary words
        // Compound words are not stored explicitly
        self.base_dict.words_iter()
    }

    /// The number of words in the dictionary.
    fn word_count(&self) -> usize {
        // Return the base dictionary count plus an estimate for compounds
        // Since we don't know the exact number of valid compounds, we return just the base count
        // This is a conservative estimate
        self.base_dict.word_count()
    }

    /// Returns the correct capitalization of the word with the given ID.
    fn get_word_from_id(&self, id: &crate::spell::WordId) -> Option<&[char]> {
        self.base_dict.get_word_from_id(id)
    }

    /// Look for words with a specific prefix
    fn find_words_with_prefix(&self, prefix: &[char]) -> Vec<Cow<'_, [char]>> {
        self.base_dict.find_words_with_prefix(prefix)
    }

    /// Look for words that share a prefix with the provided word
    fn find_words_with_common_prefix(&self, word: &[char]) -> Vec<Cow<'_, [char]>> {
        self.base_dict.find_words_with_common_prefix(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CharString;
    use crate::spell::MutableDictionary;
    use crate::spell::rune::word_list::AnnotatedWord;

    fn create_test_base_dict() -> Arc<FstDictionary> {
        let mut dict = MutableDictionary::new();

        dict.append_word("schuh".chars().collect::<CharString>(), Default::default());
        dict.append_word(
            "hersteller".chars().collect::<CharString>(),
            Default::default(),
        );
        dict.append_word("arbeit".chars().collect::<CharString>(), Default::default());

        Arc::new(dict.into())
    }

    fn create_test_compound_checker() -> CompoundChecker {
        let words = vec![
            AnnotatedWord {
                letters: "schuh".chars().collect(),
                annotations: vec!['N', 'X', 'h'],
            },
            AnnotatedWord {
                letters: "hersteller".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "arbeit".chars().collect(),
                annotations: vec!['N', 'i'],
            },
            AnnotatedWord {
                letters: "geber".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        CompoundChecker::new(&words)
    }

    #[test]
    fn test_base_dict_words_still_work() {
        let base_dict = create_test_base_dict();
        let compound_checker = create_test_compound_checker();
        let dict = CompoundAwareDictionary::new(base_dict, compound_checker);

        assert!(dict.contains_word(&"schuh".chars().collect::<Vec<_>>()));
        assert!(dict.contains_word(&"hersteller".chars().collect::<Vec<_>>()));
        assert!(dict.contains_word(&"arbeit".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_compound_words_are_recognized() {
        let base_dict = create_test_base_dict();
        let compound_checker = create_test_compound_checker();
        let dict = CompoundAwareDictionary::new(base_dict, compound_checker);

        // "schuhhersteller" should be recognized as a compound
        assert!(dict.contains_word(&"schuhhersteller".chars().collect::<Vec<_>>()));

        // "arbeitsgeber" should be recognized as a compound
        assert!(dict.contains_word(&"arbeitsgeber".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_non_compound_words_rejected() {
        let base_dict = create_test_base_dict();
        let compound_checker = create_test_compound_checker();
        let dict = CompoundAwareDictionary::new(base_dict, compound_checker);

        // "xyzabc" should not be recognized
        assert!(!dict.contains_word(&"xyzabc".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_compound_metadata() {
        let base_dict = create_test_base_dict();
        let compound_checker = create_test_compound_checker();
        let dict = CompoundAwareDictionary::new(base_dict, compound_checker);

        // Base word should return None for metadata (since we didn't set any)
        let base_metadata = dict.get_word_metadata(&"schuh".chars().collect::<Vec<_>>());
        assert!(base_metadata.is_none());

        // Compound word should return compound metadata
        let compound_metadata =
            dict.get_word_metadata(&"schuhhersteller".chars().collect::<Vec<_>>());
        assert!(compound_metadata.is_some());
    }

    #[test]
    fn test_string_versions() {
        let base_dict = create_test_base_dict();
        let compound_checker = create_test_compound_checker();
        let dict = CompoundAwareDictionary::new(base_dict, compound_checker);

        assert!(dict.contains_word_str("schuh"));
        assert!(dict.contains_word_str("schuhhersteller"));
    }
}
