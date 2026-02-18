use std::borrow::Cow;

use blanket::blanket;

use super::FuzzyMatchResult;
use crate::{DictWordMetadata, spell::word_map::WordMapEntry};

/// An in-memory database that contains everything necessary to parse and analyze English text.
///
/// See also: [`super::FstDictionary`] and [`super::MutableDictionary`].
#[blanket(derive(Arc, Ref))]
pub trait Dictionary: Send + Sync {
    /// Check if the dictionary contains any capitalization of a given word.
    fn contains_word(&self, word: &[char]) -> bool;

    /// Check if the dictionary contains the exact capitalization of a given word.
    fn contains_exact_word(&self, word: &[char]) -> bool;

    /// Gets best fuzzy match from dictionary
    fn fuzzy_match(
        &'_ self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>>;

    /// Get the correct canonical capitalizations for the given word.
    fn get_correct_capitalizations_of(&self, word: &[char]) -> Vec<&'_ [char]> {
        self.get_word(word)
            .into_iter()
            .map(|word| word.canonical_spelling.as_slice())
            .collect()
    }

    /// Get the associated [`DictWordMetadata`] for any capitalization of a given word.
    ///
    /// Since the dictionary might contain words that differ only in capitalization, this may
    /// return multiple entries.
    fn get_word(&self, word: &[char]) -> Vec<&WordMapEntry>;

    /// Get the associated [`DictWordMetadata`] for this specific capitalization of the given word.
    fn get_word_exact(&self, word: &[char]) -> Option<&WordMapEntry>;

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
        let found_words = self.get_word(word);

        match found_words.len() {
            0 => None,
            1 => Some(Cow::Borrowed(&found_words[0].metadata)),
            _ => Some(Cow::Owned({
                let mut first = found_words[0].to_owned().metadata;
                found_words.iter().skip(1).for_each(|found_word| {
                    first.append(&found_word.metadata);
                });
                first
            })),
        }
    }

    // STRING FUNCTION VARIANTS START

    /// Check if the dictionary contains any capitalization of a given word.
    fn contains_word_str(&self, word: &str) -> bool {
        self.contains_word(str_to_chars(word).as_ref())
    }

    /// Check if the dictionary contains the exact capitalization of a given word.
    fn contains_exact_word_str(&self, word: &str) -> bool {
        self.contains_exact_word(str_to_chars(word).as_ref())
    }

    /// Gets best fuzzy match from dictionary
    fn fuzzy_match_str(
        &'_ self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>> {
        self.fuzzy_match(str_to_chars(word).as_ref(), max_distance, max_results)
    }

    /// Get the associated [`DictWordMetadata`] for any capitalization of a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    ///
    /// Since the dictionary might contain words that differ only in capitalization, this may
    /// return multiple entries.
    fn get_word_str(&self, word: &str) -> Vec<&WordMapEntry> {
        self.get_word(str_to_chars(word).as_ref())
    }

    /// Get the associated [`DictWordMetadata`] for this specific capitalization of the given word.
    fn get_word_exact_str(&self, word: &str) -> Option<&WordMapEntry> {
        self.get_word_exact(str_to_chars(word).as_ref())
    }

    /// Search for a word's metadata case-insensitively, then merge all the results into one
    /// [`DictWordMetadata`].
    fn get_word_metadata_combined_str(&self, word: &str) -> Option<Cow<'_, DictWordMetadata>> {
        self.get_word_metadata_combined(str_to_chars(word).as_ref())
    }

    // STRING FUNCTION VARIANTS END
}

/// The default conversion function for converting a str to a sequence of characters.
///
/// For use by default implementations of the "str variants" of dictionary functions.
fn str_to_chars(word: &str) -> impl AsRef<[char]> {
    word.chars().collect::<Vec<_>>()
}
