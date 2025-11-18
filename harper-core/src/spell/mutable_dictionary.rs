use super::{
    FstDictionary, WordId,
    rune::{self, AttributeList, parse_word_list},
    word_map::{WordMap, WordMapEntry},
};
use crate::edit_distance::edit_distance_min_alloc;
use itertools::Itertools;
use lazy_static::lazy_static;
use std::borrow::Cow;
use std::sync::Arc;

use crate::{CharString, CharStringExt, DictWordMetadata};

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

/// The uncached function that is used to produce the original copy of the
/// curated dictionary.
fn uncached_inner_new() -> Arc<MutableDictionary> {
    MutableDictionary::from_rune_files(
        include_str!("../../dictionary.dict"),
        include_str!("../../annotations.json"),
    )
    .map(Arc::new)
    .unwrap_or_else(|e| panic!("Failed to load curated dictionary: {}", e))
}

lazy_static! {
    static ref DICT: Arc<MutableDictionary> = uncached_inner_new();
}

impl MutableDictionary {
    pub fn new() -> Self {
        Self {
            word_map: WordMap::default(),
        }
    }

    pub fn from_rune_files(word_list: &str, attr_list: &str) -> Result<Self, rune::Error> {
        let word_list = parse_word_list(word_list)?;
        let attr_list = AttributeList::parse(attr_list)?;

        // There will be at _least_ this number of words
        let mut word_map = WordMap::default();

        attr_list.expand_annotated_words(word_list, &mut word_map);

        Ok(Self { word_map })
    }

    /// Create a dictionary from the curated dictionary included
    /// in the Harper binary.
    /// Consider using [`super::FstDictionary::curated()`] instead, as it is more performant for spellchecking.
    pub fn curated() -> Arc<Self> {
        (*DICT).clone()
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
    fn get_word_metadata(&self, word: &[char]) -> Option<Cow<'_, DictWordMetadata>> {
        self.word_map
            .get_with_chars(word)
            .map(|v| Cow::Borrowed(&v.metadata))
    }

    fn contains_word(&self, word: &[char]) -> bool {
        self.word_map.contains_chars(word)
    }

    fn contains_word_str(&self, word: &str) -> bool {
        let chars: CharString = word.chars().collect();
        self.contains_word(&chars)
    }

    fn get_word_metadata_str(&self, word: &str) -> Option<Cow<'_, DictWordMetadata>> {
        let chars: CharString = word.chars().collect();
        self.get_word_metadata(&chars)
    }

    fn get_correct_capitalization_of(&self, word: &[char]) -> Option<&'_ [char]> {
        self.word_map
            .get_with_chars(word)
            .map(|v| v.canonical_spelling.as_slice())
    }

    /// Suggest a correct spelling for a given misspelled word.
    /// `Self::word` is assumed to be quite small (n < 100).
    /// `max_distance` relates to an optimization that allows the search
    /// algorithm to prune large portions of the search.
    fn fuzzy_match(
        &'_ self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>> {
        let misspelled_charslice = word.normalized();
        let misspelled_charslice_lower = misspelled_charslice.to_lower();

        let shortest_word_len = if misspelled_charslice.len() <= max_distance as usize {
            1
        } else {
            misspelled_charslice.len() - max_distance as usize
        };
        let longest_word_len = misspelled_charslice.len() + max_distance as usize;

        // Get candidate words
        let words_to_search = self
            .words_iter()
            .filter(|word| (shortest_word_len..=longest_word_len).contains(&word.len()));

        // Pre-allocated vectors for the edit-distance calculation
        // 53 is the length of the longest word.
        let mut buf_a = Vec::with_capacity(53);
        let mut buf_b = Vec::with_capacity(53);

        // Sort by edit-distance
        words_to_search
            .filter_map(|word| {
                let dist =
                    edit_distance_min_alloc(&misspelled_charslice, word, &mut buf_a, &mut buf_b);
                let lowercase_dist = edit_distance_min_alloc(
                    &misspelled_charslice_lower,
                    word,
                    &mut buf_a,
                    &mut buf_b,
                );

                let smaller_dist = dist.min(lowercase_dist);
                if smaller_dist <= max_distance {
                    Some((word, smaller_dist))
                } else {
                    None
                }
            })
            .sorted_unstable_by_key(|a| a.1)
            .take(max_results)
            .map(|(word, edit_distance)| FuzzyMatchResult {
                word,
                edit_distance,
                metadata: self.get_word_metadata(word).unwrap(),
            })
            .collect()
    }

    fn fuzzy_match_str(
        &'_ self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>> {
        let word: Vec<_> = word.chars().collect();
        self.fuzzy_match(&word, max_distance, max_results)
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        Box::new(
            self.word_map
                .iter()
                .map(|v| v.canonical_spelling.as_slice()),
        )
    }

    fn word_count(&self) -> usize {
        self.word_map.len()
    }

    fn contains_exact_word(&self, word: &[char]) -> bool {
        let normalized = word.normalized();

        if let Some(found) = self.word_map.get_with_chars(normalized.as_ref())
            && found.canonical_spelling.as_ref() == normalized.as_ref()
        {
            return true;
        }

        false
    }

    fn contains_exact_word_str(&self, word: &str) -> bool {
        let word: CharString = word.chars().collect();
        self.contains_exact_word(word.as_ref())
    }

    fn get_word_from_id(&self, id: &WordId) -> Option<&[char]> {
        self.word_map.get(id).map(|w| w.canonical_spelling.as_ref())
    }

    fn find_words_with_prefix(&self, prefix: &[char]) -> Vec<Cow<'_, [char]>> {
        let mut found = Vec::new();

        for word in self.words_iter() {
            if let Some(item_prefix) = word.get(0..prefix.len())
                && item_prefix == prefix
            {
                found.push(Cow::Borrowed(word));
            }
        }

        found
    }

    fn find_words_with_common_prefix(&self, word: &[char]) -> Vec<Cow<'_, [char]>> {
        let mut found = Vec::new();

        for item in self.words_iter() {
            if let Some(item_prefix) = word.get(0..item.len())
                && item_prefix == item
            {
                found.push(Cow::Borrowed(item));
            }
        }

        found
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
        assert!(dict.get_word_metadata_str("this").unwrap().is_determiner());
        assert!(dict.get_word_metadata_str("This").unwrap().is_determiner());
    }

    #[test]
    fn several_is_quantifier() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_metadata_str("several")
                .unwrap()
                .is_quantifier()
        );
    }

    #[test]
    fn few_is_quantifier() {
        let dict = MutableDictionary::curated();
        assert!(dict.get_word_metadata_str("few").unwrap().is_quantifier());
    }

    #[test]
    fn fewer_is_quantifier() {
        let dict = MutableDictionary::curated();
        assert!(dict.get_word_metadata_str("fewer").unwrap().is_quantifier());
    }

    #[test]
    fn than_is_conjunction() {
        let dict = MutableDictionary::curated();
        assert!(dict.get_word_metadata_str("than").unwrap().is_conjunction());
        assert!(dict.get_word_metadata_str("Than").unwrap().is_conjunction());
    }

    #[test]
    fn herself_is_pronoun() {
        let dict = MutableDictionary::curated();
        assert!(dict.get_word_metadata_str("herself").unwrap().is_pronoun());
        assert!(dict.get_word_metadata_str("Herself").unwrap().is_pronoun());
    }

    #[test]
    fn discussion_171() {
        let dict = MutableDictionary::curated();
        assert!(dict.contains_word_str("natively"));
    }

    #[test]
    fn im_is_common() {
        let dict = MutableDictionary::curated();
        assert!(dict.get_word_metadata_str("I'm").unwrap().common);
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

        assert!(!dict.get_word_metadata_str("there").unwrap().is_nominal());
        assert!(!dict.get_word_metadata_str("there").unwrap().is_pronoun());
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
                .get_word_metadata_str("abandonment's")
                .unwrap()
                .is_possessive_noun()
        )
    }

    #[test]
    fn has_is_not_a_nominal() {
        let dict = MutableDictionary::curated();

        let has = dict.get_word_metadata_str("has");
        assert!(has.is_some());

        assert!(!has.unwrap().is_nominal())
    }

    #[test]
    fn is_is_linking_verb() {
        let dict = MutableDictionary::curated();

        let is = dict.get_word_metadata_str("is");

        assert!(is.is_some());
        assert!(is.unwrap().is_linking_verb());
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

        assert!(!dict.get_word_metadata_str("apart").unwrap().is_noun());
    }

    #[test]
    fn be_is_verb_lemma() {
        let dict = MutableDictionary::curated();

        let is = dict.get_word_metadata_str("be");

        assert!(is.is_some());
        assert!(is.unwrap().is_verb_lemma());
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
