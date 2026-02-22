use std::borrow::Cow;
use std::hash::{BuildHasher, Hasher};
use std::sync::Arc;

use foldhash::quality::FixedState;
use itertools::Itertools;

use super::FstDictionary;
use super::{FuzzyMatchResult, dictionary::Dictionary};
use crate::spell::{CommonDictFuncs, WordMap};

/// A simple wrapper over [`Dictionary`] that allows
/// one to merge multiple dictionaries without copying.
///
/// In cases where more than one dictionary contains a word, data in the first
/// dictionary inserted will be returned.
#[derive(Clone, Default)]
pub struct MergedDictionary {
    merged_word_map: WordMap,
    children: Vec<Arc<dyn Dictionary>>,
    hasher_builder: FixedState,
    child_hashes: Vec<u64>,
}

impl MergedDictionary {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_dictionary(&mut self, dictionary: Arc<dyn Dictionary>) {
        self.child_hashes.push(self.hash_dictionary(&dictionary));
        self.merged_word_map
            .extend(dictionary.get_word_map().clone());
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

impl Dictionary for MergedDictionary {
    fn get_word_map(&self) -> &WordMap {
        &self.merged_word_map
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
