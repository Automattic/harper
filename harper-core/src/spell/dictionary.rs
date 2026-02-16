use std::borrow::Cow;

use blanket::blanket;

use super::FuzzyMatchResult;
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

    /// Get the correct canonical capitalizations for the given word.
    fn get_correct_capitalization_of(&self, word: &[char]) -> Vec<&'_ [char]>;

    /// Get the associated [`DictWordMetadata`] for any capitalization of a given word.
    ///
    /// Since the dictionary might contain words that differ only in capitalization, this may
    /// return multiple entries.
    fn get_word_metadata(&self, word: &[char]) -> Vec<&DictWordMetadata>;

    /// Get the associated [`DictWordMetadata`] for this specific capitalization of the given word.
    fn get_word_metadata_exact(&self, word: &[char]) -> Option<&DictWordMetadata>;

    /// Get the associated [`DictWordMetadata`] for any capitalization of a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    ///
    /// Since the dictionary might contain words that differ only in capitalization, this may
    /// return multiple entries.
    fn get_word_metadata_str(&self, word: &str) -> Vec<&DictWordMetadata>;

    /// Get the associated [`DictWordMetadata`] for this specific capitalization of the given word.
    fn get_word_metadata_str_exact(&self, word: &str) -> Option<&DictWordMetadata>;

    /// Iterate over the words in the dictionary.
    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_>;

    /// The number of words in the dictionary.
    fn word_count(&self) -> usize;

    /// Look for words with a specific prefix
    fn find_words_with_prefix(&self, prefix: &[char]) -> Vec<Cow<'_, [char]>>;

    /// Look for words that share a prefix with the provided word
    fn find_words_with_common_prefix(&self, word: &[char]) -> Vec<Cow<'_, [char]>>;

    /// Search for a word's metadata case-insensitively, then merge all the results into one
    /// [`DictWordMetadata`].
    fn get_word_metadata_combined(&self, word: &[char]) -> Option<Cow<'_, DictWordMetadata>> {
        let found_words = self.get_word_metadata(word);

        match found_words.len() {
            0 => None,
            1 => Some(Cow::Borrowed(found_words[0])),
            _ => Some(Cow::Owned({
                let mut first = found_words[0].to_owned();
                found_words.iter().skip(1).for_each(|found_word| {
                    first.append(found_word);
                });
                first
            })),
        }
    }

    /// Search for a word's metadata case-insensitively, then merge all the results into one
    /// [`DictWordMetadata`].
    fn get_word_metadata_combined_str(&self, word: &str) -> Option<Cow<'_, DictWordMetadata>> {
        self.get_word_metadata_combined(&word.chars().collect::<Vec<_>>())
    }
}
