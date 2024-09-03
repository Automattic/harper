use blanket::blanket;

use crate::WordMetadata;

#[cfg(not(feature = "concurrent"))]
#[blanket(derive(Rc))]
pub trait Dictionary: Clone {
    /// Check if the dictionary contains a given word.
    fn contains_word(&self, word: &[char]) -> bool;
    /// Check if the dictionary contains a given word.
    fn contains_word_str(&self, word: &str) -> bool;
    /// Produce an iterator over all words in the dictionary.
    fn words_iter(&self) -> impl Iterator<Item = &'_ [char]>;
    /// Iterate over all the words in the dictionary of a given length
    fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + '_>;
    /// Get the associated [`WordMetadata`] for a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    fn get_word_metadata(&self, word: &[char]) -> WordMetadata;
    /// Get the associated [`WordMetadata`] for a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    fn get_word_metadata_str(&self, word: &str) -> WordMetadata;
}

#[cfg(feature = "concurrent")]
#[blanket(derive(Arc))]
pub trait Dictionary: Send + Sync + Clone {
    /// Check if the dictionary contains a given word.
    fn contains_word(&self, word: &[char]) -> bool;
    /// Check if the dictionary contains a given word.
    fn contains_word_str(&self, word: &str) -> bool;
    /// Produce an iterator over all words in the dictionary.
    fn words_iter(&self) -> impl Iterator<Item = &'_ [char]>;
    /// Iterate over all the words in the dictionary of a given length
    fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + '_>;
    /// Get the associated [`WordMetadata`] for a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    fn get_word_metadata(&self, word: &[char]) -> WordMetadata;
    /// Get the associated [`WordMetadata`] for a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    fn get_word_metadata_str(&self, word: &str) -> WordMetadata;
}
