use super::{
    FstDictionary, rune,
    word_map::{self, WordMap, WordMapEntry},
};
use std::{borrow::Cow, sync::LazyLock};

use crate::DictWordMetadata;

use super::FuzzyMatchResult;
use super::dictionary::Dictionary;

/// A basic dictionary that allows words to be added after instantiating.
/// This is useful for user and file dictionaries that may change at runtime.
///
/// For immutable use-cases that require fuzzy lookups, such as the curated dictionary, prefer [`super::FstDictionary`],
/// as it is much faster.
///
/// To combine the contents of multiple dictionaries, regardless of type, use
/// [`super::MergedDictionary`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MutableDictionary {
    /// All English words
    word_map: WordMap,
}

impl MutableDictionary {
    pub fn new() -> Self {
        Self {
            word_map: WordMap::default(),
        }
    }

    pub fn from_rune_files(word_list: &str, attr_list: &str) -> Result<Self, rune::Error> {
        Ok(Self {
            word_map: WordMap::from_rune_files(word_list, attr_list)?,
        })
    }

    /// Create a dictionary from the curated dictionary included
    /// in the Harper binary.
    /// Consider using [`super::FstDictionary::curated()`] instead, as it is more performant for spellchecking.
    pub fn curated() -> &'static Self {
        static DICT: LazyLock<MutableDictionary> = LazyLock::new(|| MutableDictionary {
            word_map: word_map::CURATED.clone(),
        });

        &DICT
    }

    /// Appends words to the dictionary.
    /// It is significantly faster to append many words with one call than many
    /// distinct calls to this function.
    pub fn extend_words(
        &mut self,
        words: impl IntoIterator<Item = (impl AsRef<[char]>, DictWordMetadata)>,
    ) {
        for (chars, metadata) in words.into_iter() {
            self.word_map.insert(WordMapEntry {
                metadata,
                canonical_spelling: chars.as_ref().into(),
            })
        }
    }

    /// Append a single word to the dictionary.
    ///
    /// If you are appending many words, consider using [`Self::extend_words`]
    /// instead.
    pub fn append_word(&mut self, word: impl AsRef<[char]>, metadata: DictWordMetadata) {
        self.extend_words(std::iter::once((word.as_ref(), metadata)))
    }

    /// Append a single string to the dictionary.
    ///
    /// If you are appending many words, consider using [`Self::extend_words`]
    /// instead.
    pub fn append_word_str(&mut self, word: &str, metadata: DictWordMetadata) {
        self.append_word(word.chars().collect::<Vec<_>>(), metadata)
    }
}

impl Default for MutableDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl Dictionary for MutableDictionary {
    fn contains_word(&self, word: &[char]) -> bool {
        self.word_map.contains_word(word)
    }

    fn contains_exact_word(&self, word: &[char]) -> bool {
        self.word_map.contains_exact_word(word)
    }

    fn fuzzy_match(
        &'_ self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>> {
        self.word_map.fuzzy_match(word, max_distance, max_results)
    }

    fn get_word(&self, word: &[char]) -> Vec<&WordMapEntry> {
        self.word_map.get_word(word)
    }

    fn get_word_exact(&self, word: &[char]) -> Option<&WordMapEntry> {
        self.word_map.get_word_exact(word)
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        self.word_map.words_iter()
    }

    fn word_count(&self) -> usize {
        self.word_map.word_count()
    }

    fn find_words_with_prefix(&self, prefix: &[char]) -> Vec<Cow<'_, [char]>> {
        self.word_map.find_words_with_prefix(prefix)
    }

    fn find_words_with_common_prefix(&self, word: &[char]) -> Vec<Cow<'_, [char]>> {
        self.word_map.find_words_with_common_prefix(word)
    }
}

impl From<MutableDictionary> for FstDictionary {
    fn from(dict: MutableDictionary) -> Self {
        let words = dict
            .word_map
            .into_iter()
            .map(|entry| (entry.canonical_spelling, entry.metadata))
            .collect();

        FstDictionary::new(words)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use hashbrown::HashSet;
    use itertools::Itertools;

    use crate::spell::{Dictionary, MutableDictionary};
    use crate::{DictWordMetadata, char_string::char_string};

    #[test]
    fn curated_contains_no_duplicates() {
        let dict = MutableDictionary::curated();
        assert!(dict.words_iter().all_unique());
    }

    #[test]
    fn curated_matches_capitalized() {
        let dict = MutableDictionary::curated();
        assert!(dict.contains_word_str("this"));
        assert!(dict.contains_word_str("This"));
    }

    // "This" is a determiner when used similarly to "the"
    // but when used alone it's a "demonstrative pronoun".
    // Harper previously wrongly classified it as a noun.
    #[test]
    fn this_is_determiner() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("this")
                .unwrap()
                .metadata
                .is_determiner()
        );
    }

    #[test]
    fn several_is_quantifier() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("several")
                .unwrap()
                .metadata
                .is_quantifier()
        );
    }

    #[test]
    fn few_is_quantifier() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("few")
                .unwrap()
                .metadata
                .is_quantifier()
        );
    }

    #[test]
    fn fewer_is_quantifier() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("fewer")
                .unwrap()
                .metadata
                .is_quantifier()
        );
    }

    #[test]
    fn than_is_conjunction() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("than")
                .unwrap()
                .metadata
                .is_conjunction()
        );
    }

    #[test]
    fn herself_is_pronoun() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("herself")
                .unwrap()
                .metadata
                .is_pronoun()
        );
    }

    #[test]
    fn discussion_171() {
        let dict = MutableDictionary::curated();
        assert!(dict.contains_word_str("natively"));
    }

    #[test]
    fn im_is_common() {
        let dict = MutableDictionary::curated();
        assert!(dict.get_word_exact_str("I'm").unwrap().metadata.common);
    }

    #[test]
    fn fuzzy_result_sorted_by_edit_distance() {
        let dict = MutableDictionary::curated();

        let results = dict.fuzzy_match_str("hello", 3, 100);
        let is_sorted_by_dist = results
            .iter()
            .map(|fm| fm.edit_distance)
            .tuple_windows()
            .all(|(a, b)| a <= b);

        assert!(is_sorted_by_dist)
    }

    #[test]
    fn there_is_not_a_pronoun() {
        let dict = MutableDictionary::curated();
        let there_meta = dict.get_word_exact_str("there").unwrap();

        assert!(!there_meta.metadata.is_nominal());
        assert!(!there_meta.metadata.is_pronoun());
    }

    #[test]
    fn expanded_contains_giants() {
        assert!(MutableDictionary::curated().contains_word_str("giants"));
    }

    #[test]
    fn expanded_contains_deallocate() {
        assert!(MutableDictionary::curated().contains_word_str("deallocate"));
    }

    #[test]
    fn curated_contains_repo() {
        let dict = MutableDictionary::curated();

        assert!(dict.contains_word_str("repo"));
        assert!(dict.contains_word_str("repos"));
        assert!(dict.contains_word_str("repo's"));
    }

    #[test]
    fn curated_contains_possessive_abandonment() {
        assert!(
            MutableDictionary::curated()
                .get_word_exact_str("abandonment's")
                .unwrap()
                .metadata
                .is_possessive_noun()
        )
    }

    #[test]
    fn has_is_not_a_nominal() {
        let dict = MutableDictionary::curated();

        let has = dict.get_word_exact_str("has");
        assert!(has.is_some());

        assert!(!has.unwrap().metadata.is_nominal())
    }

    #[test]
    fn is_is_linking_verb() {
        let dict = MutableDictionary::curated();

        let is = dict.get_word_exact_str("is");

        assert!(is.is_some());
        assert!(is.unwrap().metadata.is_linking_verb());
    }

    #[test]
    fn are_merged_attrs_same_as_spread_attrs() {
        let curated_attr_list = include_str!("../../annotations.json");

        let merged = MutableDictionary::from_rune_files("1\nblork/DGS", curated_attr_list).unwrap();
        let spread =
            MutableDictionary::from_rune_files("2\nblork/DG\nblork/S", curated_attr_list).unwrap();

        assert_eq!(
            merged.word_map.into_iter().collect::<HashSet<_>>(),
            spread.word_map.into_iter().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn apart_is_not_noun() {
        let dict = MutableDictionary::curated();

        assert!(!dict.get_word_exact_str("apart").unwrap().metadata.is_noun());
    }

    #[test]
    fn be_is_verb_lemma() {
        let dict = MutableDictionary::curated();

        let is = dict.get_word_exact_str("be");

        assert!(is.is_some());
        assert!(is.unwrap().metadata.is_verb_lemma());
    }

    #[test]
    fn gets_prefixes_as_expected() {
        let mut dict = MutableDictionary::new();
        dict.append_word_str("predict", DictWordMetadata::default());
        dict.append_word_str("prelude", DictWordMetadata::default());
        dict.append_word_str("preview", DictWordMetadata::default());
        dict.append_word_str("dwight", DictWordMetadata::default());

        let with_prefix = dict.find_words_with_prefix(char_string!("pre").as_slice());

        assert_eq!(with_prefix.len(), 3);
        assert!(with_prefix.contains(&Cow::Owned(char_string!("predict").into_vec())));
        assert!(with_prefix.contains(&Cow::Owned(char_string!("prelude").into_vec())));
        assert!(with_prefix.contains(&Cow::Owned(char_string!("preview").into_vec())));
    }

    #[test]
    fn gets_common_prefixes_as_expected() {
        let mut dict = MutableDictionary::new();
        dict.append_word_str("pre", DictWordMetadata::default());
        dict.append_word_str("prep", DictWordMetadata::default());
        dict.append_word_str("dwight", DictWordMetadata::default());

        let with_prefix =
            dict.find_words_with_common_prefix(char_string!("preposition").as_slice());

        assert_eq!(with_prefix.len(), 2);
        assert!(with_prefix.contains(&Cow::Owned(char_string!("pre").into_vec())));
        assert!(with_prefix.contains(&Cow::Owned(char_string!("prep").into_vec())));
    }
}
