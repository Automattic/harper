use std::borrow::Cow;

use blanket::blanket;

use super::FuzzyMatchResult;
use crate::{
    DictWordMetadata,
    spell::{CanonicalWordId, CaseFoldedWordId, WordMap, WordMapEntry},
};

pub(crate) static CURATED_DICT_STR: &str = include_str!("../../dictionary.dict");
pub(crate) static ANNOTATIONS_STR: &str = include_str!("../../annotations.json");

/// An in-memory database that contains everything necessary to parse and analyze English text.
///
/// See also: [`super::FstDictionary`] and [`super::MutableDictionary`].
#[blanket(derive(Arc, Ref))]
pub trait Dictionary: Send + Sync {
    /// Get the underlying [`WordMap`] used by the dictionary.
    fn get_word_map(&self) -> &WordMap;

    /// Gets best fuzzy match from dictionary
    fn fuzzy_match(
        &'_ self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>>;

    /// Look for words with a specific prefix
    fn find_words_with_prefix(&self, prefix: &[char]) -> Vec<Cow<'_, [char]>>;

    /// Look for words that share a prefix with the provided word
    fn find_words_with_common_prefix(&self, word: &[char]) -> Vec<Cow<'_, [char]>>;

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
    // STRING FUNCTION VARIANTS END
}

/// The default conversion function for converting a str to a sequence of characters.
///
/// For use by default implementations of the "str variants" of dictionary functions.
fn str_to_chars(word: &str) -> impl AsRef<[char]> {
    word.chars().collect::<Vec<_>>()
}

// This trait doesn't need to be dyn-compatible, as such, it can use things like generics.
/// Contains functions which have a common implementation across all dictionaries.
pub trait CommonDictFuncs: Dictionary {
    /// Get the associated [`DictWordMetadata`] for any capitalization of a given word.
    ///
    /// Since the dictionary might contain words that differ only in capitalization, this may
    /// return multiple entries.
    fn get_word(
        &self,
        word: &[char],
    ) -> impl ExactSizeIterator<Item = &WordMapEntry> + use<'_, Self> {
        self.get_word_map().get_case_folded_chars(word)
    }

    /// Get the associated [`DictWordMetadata`] for this specific capitalization of the given word.
    fn get_word_exact(&self, word: &[char]) -> Option<&WordMapEntry> {
        self.get_word_map()
            .get_canonical(CanonicalWordId::from_word_chars(word))
    }

    /// Search for a word's metadata case-insensitively, then merge all the results into one
    /// [`DictWordMetadata`].
    fn get_word_metadata_combined(&self, word: &[char]) -> Option<Cow<'_, DictWordMetadata>> {
        let mut found_words = self.get_word(word);

        match found_words.len() {
            0 => None,
            1 => Some(Cow::Borrowed(&found_words.next().unwrap().metadata)),
            _ => Some(Cow::Owned({
                let mut first = found_words.next().unwrap().metadata.to_owned();
                found_words.for_each(|found_word| {
                    first.append(&found_word.metadata);
                });
                first
            })),
        }
    }

    /// Check if the dictionary contains any capitalization of a given word.
    fn contains_word(&self, word: &[char]) -> bool {
        self.get_word_map()
            .contains_case_folded(CaseFoldedWordId::from_word_chars(word).0)
    }

    /// Check if the dictionary contains the exact capitalization of a given word.
    fn contains_exact_word(&self, word: &[char]) -> bool {
        self.get_word_map()
            .contains_canonical(CanonicalWordId::from_word_chars(word))
    }

    /// The number of words in the dictionary.
    fn word_count(&self) -> usize {
        self.get_word_map().len()
    }

    /// Iterate over the words in the dictionary.
    fn words_iter(&self) -> impl ExactSizeIterator<Item = &[char]> {
        self.get_word_map()
            .iter()
            .map(|wme| wme.canonical_spelling.as_slice())
    }

    /// Get the correct canonical capitalizations for the given word.
    fn get_correct_capitalizations_of(
        &self,
        word: &[char],
    ) -> impl ExactSizeIterator<Item = &[char]> + use<'_, Self> {
        self.get_word(word)
            .map(|word| word.canonical_spelling.as_slice())
    }

    // STRING FUNCTION VARIANTS START
    /// Get the associated [`DictWordMetadata`] for any capitalization of a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    ///
    /// Since the dictionary might contain words that differ only in capitalization, this may
    /// return multiple entries.
    fn get_word_str(
        &self,
        word: &str,
    ) -> impl ExactSizeIterator<Item = &WordMapEntry> + use<'_, Self> {
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

impl<D: Dictionary> CommonDictFuncs for D {}
