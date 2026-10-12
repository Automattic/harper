use blanket::blanket;
use std::borrow::Cow;

use super::FuzzyMatchResult;
use super::WordId;
use crate::DictWordMetadata;

/// An in-memory database that contains everything necessary to parse and analyze English text.
///
/// See also: [`super::FstDictionary`] and [`super::MutableDictionary`].
#[blanket(derive(Arc, Ref))]
pub trait Dictionary: Send + Sync {
    /// Check if the dictionary contains any capitalization of a given word.
    fn contains_word(&self, word: &[char]) -> bool;
    /// Check if the dictionary contains any capitalization of a given word.
    fn contains_word_str(&self, word: &str) -> bool;
    /// Check if the dictionary contains the exact capitalization of a given word.
    fn contains_exact_word(&self, word: &[char]) -> bool;
    /// Check if the dictionary contains the exact capitalization of a given word.
    fn contains_exact_word_str(&self, word: &str) -> bool;
    /// Gets best fuzzy match from dictionary
    fn fuzzy_match(
        &'_ self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>>;
    /// Gets best fuzzy match from dictionary
    fn fuzzy_match_str(
        &'_ self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>>;
    fn get_correct_capitalization_of(&self, word: &[char]) -> Option<&'_ [char]>;
    /// Get the associated [`DictWordMetadata`] for any capitalization of a given word.
    fn get_word_metadata(&self, word: &[char]) -> Option<Cow<'_, DictWordMetadata>>;
    /// Get the associated [`DictWordMetadata`] for any capitalization of a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    fn get_word_metadata_str(&self, word: &str) -> Option<Cow<'_, DictWordMetadata>>;

    /// Iterate over the words in the dictionary.
    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_>;

    /// The number of words in the dictionary.
    fn word_count(&self) -> usize;

    /// Returns the correct capitalization of the word with the given ID.
    fn get_word_from_id(&self, id: &WordId) -> Option<&[char]>;

    /// Look for words with a specific prefix
    fn find_words_with_prefix(&self, prefix: &[char]) -> Vec<Cow<'_, [char]>>;

    /// Look for words that share a prefix with the provided word
    fn find_words_with_common_prefix(&self, word: &[char]) -> Vec<Cow<'_, [char]>>;

    /// A hash of the words in the dictionary, if the dictionary can hand one out
    /// without recomputing it.
    ///
    /// [`MergedDictionary`](super::MergedDictionary) fingerprints every child it
    /// is given, and that walks every word. A dictionary that never changes and
    /// is large enough for the walk to matter can remember the answer here.
    /// Returning `Some` is a promise that the words will not change, and that the
    /// value is what `merged_dictionary::word_hash` gives for them.
    fn cached_word_hash(&self) -> Option<u64> {
        None
    }
}
