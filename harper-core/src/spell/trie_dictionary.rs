use std::borrow::Cow;
use std::sync::LazyLock;

use trie_rs::Trie;
use trie_rs::iter::{Keys, PrefixIter, SearchIter};

use crate::spell::word_map::WordMapEntry;

use super::{Dictionary, FstDictionary, FuzzyMatchResult, WordMap};

/// A [`Dictionary`] optimized for pre- and postfix search.
/// Wraps another dictionary to implement other operations.
pub struct TrieDictionary<D: Dictionary> {
    trie: Trie<char>,
    inner: D,
}

pub static DICT: LazyLock<TrieDictionary<&'static FstDictionary>> =
    LazyLock::new(|| TrieDictionary::new(FstDictionary::curated()));

impl TrieDictionary<&'static FstDictionary> {
    /// Create a dictionary from the curated dictionary included
    /// in the Harper binary.
    pub fn curated() -> &'static Self {
        &DICT
    }
}

impl<D: Dictionary> TrieDictionary<D> {
    pub fn new(inner: D) -> Self {
        let trie = Trie::from_iter(inner.words_iter());

        Self { inner, trie }
    }
}

impl<D: Dictionary> Dictionary for TrieDictionary<D> {
    fn get_word_map(&self) -> &WordMap {
        self.inner.get_word_map()
    }

    fn contains_word(&self, word: &[char]) -> bool {
        self.inner.contains_word(word)
    }

    fn contains_exact_word(&self, word: &[char]) -> bool {
        self.inner.contains_exact_word(word)
    }

    fn fuzzy_match(
        &'_ self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>> {
        self.inner.fuzzy_match(word, max_distance, max_results)
    }

    fn get_word(&self, word: &[char]) -> Vec<&WordMapEntry> {
        self.inner.get_word(word)
    }

    fn get_word_exact(&self, word: &[char]) -> Option<&WordMapEntry> {
        self.inner.get_word_exact(word)
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        self.inner.words_iter()
    }

    fn word_count(&self) -> usize {
        self.inner.word_count()
    }

    fn find_words_with_prefix(&self, prefix: &[char]) -> Vec<Cow<'_, [char]>> {
        let results: Keys<SearchIter<'_, char, (), Vec<char>, _>> =
            self.trie.predictive_search(prefix);
        results.map(Cow::Owned).collect()
    }

    fn find_words_with_common_prefix(&self, word: &[char]) -> Vec<Cow<'_, [char]>> {
        let results: Keys<PrefixIter<'_, char, (), Vec<char>, _>> =
            self.trie.common_prefix_search(word);
        results.map(Cow::Owned).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::DictWordMetadata;
    use crate::char_string::char_string;
    use crate::spell::MutableDictionary;
    use crate::spell::dictionary::Dictionary;
    use crate::spell::trie_dictionary::TrieDictionary;

    #[test]
    fn gets_prefixes_as_expected() {
        let mut inner = MutableDictionary::new();
        inner.append_word_str("predict", DictWordMetadata::default());
        inner.append_word_str("prelude", DictWordMetadata::default());
        inner.append_word_str("preview", DictWordMetadata::default());
        inner.append_word_str("dwight", DictWordMetadata::default());

        let dict = TrieDictionary::new(inner);

        let with_prefix = dict.find_words_with_prefix(char_string!("pre").as_slice());

        assert_eq!(with_prefix.len(), 3);
        assert!(with_prefix.contains(&Cow::Owned(char_string!("predict").into_vec())));
        assert!(with_prefix.contains(&Cow::Owned(char_string!("prelude").into_vec())));
        assert!(with_prefix.contains(&Cow::Owned(char_string!("preview").into_vec())));
    }

    #[test]
    fn gets_common_prefixes_as_expected() {
        let mut inner = MutableDictionary::new();
        inner.append_word_str("pre", DictWordMetadata::default());
        inner.append_word_str("prep", DictWordMetadata::default());
        inner.append_word_str("dwight", DictWordMetadata::default());

        let dict = TrieDictionary::new(inner);

        let with_prefix =
            dict.find_words_with_common_prefix(char_string!("preposition").as_slice());

        assert_eq!(with_prefix.len(), 2);
        assert!(with_prefix.contains(&Cow::Owned(char_string!("pre").into_vec())));
        assert!(with_prefix.contains(&Cow::Owned(char_string!("prep").into_vec())));
    }
}
