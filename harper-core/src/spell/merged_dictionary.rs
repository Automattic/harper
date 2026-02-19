use std::borrow::Cow;
use std::hash::{BuildHasher, Hasher};
use std::sync::Arc;

use foldhash::quality::FixedState;
use itertools::Itertools;

use super::FstDictionary;
use super::{FuzzyMatchResult, dictionary::Dictionary};
use crate::spell::word_map::WordMapEntry;

/// A simple wrapper over [`Dictionary`] that allows
/// one to merge multiple dictionaries without copying.
///
/// In cases where more than one dictionary contains a word, data in the first
/// dictionary inserted will be returned.
#[derive(Clone)]
pub struct MergedDictionary {
    children: Vec<Arc<dyn Dictionary>>,
    hasher_builder: FixedState,
    child_hashes: Vec<u64>,
}

impl MergedDictionary {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            hasher_builder: FixedState::default(),
            child_hashes: Vec::new(),
        }
    }

    pub fn add_dictionary(&mut self, dictionary: Arc<dyn Dictionary>) {
        self.child_hashes.push(self.hash_dictionary(&dictionary));
        self.children.push(dictionary);
    }

    fn hash_dictionary(&self, dictionary: &Arc<dyn Dictionary>) -> u64 {
        // Hashing the curated dictionary isn't super helpful and takes a long time.
        if std::ptr::eq(Arc::as_ptr(dictionary), FstDictionary::curated()) {
            return 1;
        }

        let mut hasher = self.hasher_builder.build_hasher();

        dictionary
            .words_iter()
            .for_each(|w| w.iter().for_each(|c| hasher.write_u32(*c as u32)));

        hasher.finish()
    }
}

impl PartialEq for MergedDictionary {
    fn eq(&self, other: &Self) -> bool {
        self.child_hashes == other.child_hashes
    }
}

impl Default for MergedDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl Dictionary for MergedDictionary {
    fn contains_word(&self, word: &[char]) -> bool {
        for child in &self.children {
            if child.contains_word(word) {
                return true;
            }
        }
        false
    }

    fn contains_exact_word(&self, word: &[char]) -> bool {
        for child in &self.children {
            if child.contains_exact_word(word) {
                return true;
            }
        }
        false
    }

    fn get_word(&self, word: &[char]) -> Vec<&WordMapEntry> {
        self.children
            .iter()
            .flat_map(|d| d.get_word(word))
            .collect()
    }

    fn get_word_exact(&self, word: &[char]) -> Option<&WordMapEntry> {
        for child in &self.children {
            if let Some(dict_word_metadata) = child.get_word_exact(word) {
                return Some(dict_word_metadata);
            }
        }
        None
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        Box::new(self.children.iter().flat_map(|c| c.words_iter()))
    }

    fn fuzzy_match(
        &'_ self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>> {
        self.children
            .iter()
            .flat_map(|d| d.fuzzy_match(word, max_distance, max_results))
            .sorted_by_key(|r| r.word)
            .dedup_by(|a, b| a.word == b.word)
            .sorted_by_key(|r| r.edit_distance)
            .take(max_results)
            .collect()
    }

    fn word_count(&self) -> usize {
        self.children.iter().map(|d| d.word_count()).sum()
    }

    fn find_words_with_prefix(&self, prefix: &[char]) -> Vec<Cow<'_, [char]>> {
        self.children
            .iter()
            .flat_map(|dict| dict.find_words_with_prefix(prefix))
            .sorted()
            .dedup()
            .collect()
    }

    fn find_words_with_common_prefix(&self, word: &[char]) -> Vec<Cow<'_, [char]>> {
        self.children
            .iter()
            .flat_map(|dict| dict.find_words_with_common_prefix(word))
            .sorted()
            .dedup()
            .collect()
    }
}
