use blanket::blanket;

use super::FuzzyMatchResult;
use super::WordId;
use crate::WordMetadata;

/// An in-memory database that contains everything necessary to parse and analyze English text.
///
/// See also: [`super::FstDictionary`] and [`super::MutableDictionary`].
#[blanket(derive(Arc))]
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
        &self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>>;
    /// Gets best fuzzy match from dictionary
    fn fuzzy_match_str(
        &self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>>;
    fn get_correct_capitalization_of(&self, word: &[char]) -> Option<&'_ [char]>;
    /// Get the associated [`WordMetadata`] for any capitalization of a given word.
    fn get_word_metadata(&self, word: &[char]) -> Option<&WordMetadata>;
    /// Get the associated [`WordMetadata`] for any capitalization of a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    fn get_word_metadata_str(&self, word: &str) -> Option<&WordMetadata>;

    /// Iterate over the words in the dictionary.
    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_>;

    /// The number of words in the dictionary.
    fn word_count(&self) -> usize;

    /// Returns the correct capitalization of the word with the given ID.
    fn get_word_from_id(&self, id: &WordId) -> Option<&[char]>;
}
