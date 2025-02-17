use super::{
    hunspell::{parse_default_attribute_list, parse_default_word_list},
    seq_to_normalized, MutableDictionary,
};
use fst::{map::StreamWithState, IntoStreamer, Map as FstMap, Streamer};
use hashbrown::HashMap;
use lazy_static::lazy_static;
use levenshtein_automata::{LevenshteinAutomatonBuilder, DFA};
use std::{cell::RefCell, sync::Arc};

use crate::{CharString, CharStringExt, WordMetadata};

use super::Dictionary;
use super::FuzzyMatchResult;

/// An immutable dictionary allowing for very fast spellchecking.
///
/// For dictionaries with changing contents, such as user and file dictionaries, prefer
/// [`super::MutableDictionary`].
pub struct FstDictionary {
    /// Underlying [`super::MutableDictionary`] used for everything except fuzzy finding
    full_dict: Arc<MutableDictionary>,
    /// Used for fuzzy-finding the index of words or metadata
    word_map: FstMap<Vec<u8>>,
    /// Used for fuzzy-finding the index of words or metadata
    words: Vec<(CharString, WordMetadata)>,
}

/// The uncached function that is used to produce the original copy of the
/// curated dictionary.
fn uncached_inner_new() -> Arc<FstDictionary> {
    let word_list = parse_default_word_list().unwrap();
    let attr_list = parse_default_attribute_list();

    // There will be at _least_ this number of words
    let mut word_map = HashMap::with_capacity(word_list.len());
    attr_list.expand_marked_words(word_list, &mut word_map);

    Arc::new(FstDictionary::new(word_map))
}

const EXPECTED_DISTANCE: u8 = 3;
const TRANSPOSITION_COST_ONE: bool = false;

lazy_static! {
    static ref DICT: Arc<FstDictionary> = uncached_inner_new();
}

thread_local! {
    // Builders are computationally expensive and do not depend on the word, so we store a
    // collection of builders and the associated edit distance here.
    // Currently, the edit distance we use is three, but a value that does not exist in this
    // collection will create a new builder of that distance and push it to the collection.
    static AUTOMATON_BUILDERS: RefCell<Vec<(u8, LevenshteinAutomatonBuilder)>> = RefCell::new(vec![(
        EXPECTED_DISTANCE,
        LevenshteinAutomatonBuilder::new(EXPECTED_DISTANCE, TRANSPOSITION_COST_ONE),
    )]);
}

impl PartialEq for FstDictionary {
    fn eq(&self, other: &Self) -> bool {
        self.full_dict == other.full_dict
    }
}

impl FstDictionary {
    /// Create a dictionary from the curated dictionary included
    /// in the Harper binary.
    pub fn curated() -> Arc<Self> {
        (*DICT).clone()
    }

    pub fn new(new_words: HashMap<CharString, WordMetadata>) -> Self {
        let mut words: Vec<(CharString, WordMetadata)> = new_words.into_iter().collect();
        words.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        words.dedup_by(|(a, _), (b, _)| a == b);

        let mut builder = fst::MapBuilder::memory();
        for (index, (word, _)) in words.iter().enumerate() {
            let word = word.iter().collect::<String>();
            builder
                .insert(word, index as u64)
                .expect("Insertion not in lexicographical order!");
        }

        let mut full_dict = MutableDictionary::new();
        full_dict.extend_words(words.iter().cloned());

        let fst_bytes = builder.into_inner().unwrap();
        let word_map = FstMap::new(fst_bytes).expect("Unable to build FST map.");

        FstDictionary {
            full_dict: Arc::new(full_dict),
            word_map,
            words,
        }
    }
}

fn build_dfa(max_distance: u8, query: &str) -> DFA {
    // Insert if it does not exist
    AUTOMATON_BUILDERS.with_borrow_mut(|v| {
        if !v.iter().any(|t| t.0 == max_distance) {
            v.push((
                max_distance,
                LevenshteinAutomatonBuilder::new(max_distance, TRANSPOSITION_COST_ONE),
            ));
        }
    });

    AUTOMATON_BUILDERS.with_borrow(|v| {
        v.iter()
            .find(|a| a.0 == max_distance)
            .unwrap()
            .1
            .build_dfa(query)
    })
}

/// Consumes a DFA stream and emits the index-edit distance pairs it produces.
fn stream_distances_vec(stream: &mut StreamWithState<&DFA>, dfa: &DFA) -> Vec<(u64, u8)> {
    let mut word_index_pairs = Vec::new();
    while let Some((_, v, s)) = stream.next() {
        word_index_pairs.push((v, dfa.distance(s).to_u8()));
    }

    word_index_pairs
}

impl Dictionary for FstDictionary {
    fn contains_word(&self, word: &[char]) -> bool {
        self.full_dict.contains_word(word)
    }

    fn contains_word_str(&self, word: &str) -> bool {
        self.full_dict.contains_word_str(word)
    }

    fn get_word_metadata(&self, word: &[char]) -> WordMetadata {
        self.full_dict.get_word_metadata(word)
    }

    fn get_word_metadata_str(&self, word: &str) -> WordMetadata {
        self.full_dict.get_word_metadata_str(word)
    }

    fn fuzzy_match(
        &self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult> {
        let misspelled_word_charslice = seq_to_normalized(word);
        let misspelled_word_string = misspelled_word_charslice.to_string();

        // Actual FST search
        let dfa = build_dfa(max_distance, &misspelled_word_string);
        let dfa_lowercase = build_dfa(max_distance, &misspelled_word_string.to_lowercase());
        let mut word_indexes_stream = self.word_map.search_with_state(&dfa).into_stream();
        let mut word_indexes_lowercase_stream = self
            .word_map
            .search_with_state(&dfa_lowercase)
            .into_stream();

        let upper_dists = stream_distances_vec(&mut word_indexes_stream, &dfa);
        let lower_dists = stream_distances_vec(&mut word_indexes_lowercase_stream, &dfa_lowercase);

        let mut merged = Vec::with_capacity(upper_dists.len());

        // Merge the two results
        for ((i_u, dist_u), (i_l, dist_l)) in upper_dists.into_iter().zip(lower_dists.into_iter()) {
            let (chosen_index, edit_distance) = if dist_u <= dist_l {
                (i_u, dist_u)
            } else {
                (i_l, dist_l)
            };

            let (word, metadata) = &self.words[chosen_index as usize];

            merged.push(FuzzyMatchResult {
                word,
                edit_distance,
                metadata: *metadata,
            })
        }

        merged.sort_unstable_by_key(|v| v.word);
        merged.dedup_by_key(|v| v.word);
        merged.sort_unstable_by_key(|v| v.edit_distance);
        merged.truncate(max_results);

        merged
    }

    fn fuzzy_match_str(
        &self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult> {
        self.fuzzy_match(
            word.chars().collect::<Vec<_>>().as_slice(),
            max_distance,
            max_results,
        )
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        self.full_dict.words_iter()
    }

    fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        self.full_dict.words_with_len_iter(len)
    }

    fn contains_exact_word(&self, word: &[char]) -> bool {
        self.full_dict.contains_exact_word(word)
    }

    fn contains_exact_word_str(&self, word: &str) -> bool {
        self.full_dict.contains_exact_word_str(word)
    }

    fn get_correct_capitalization_of(&self, word: &[char]) -> Option<&'_ [char]> {
        self.full_dict.get_correct_capitalization_of(word)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::CharStringExt;
    use crate::{spell::seq_to_normalized, Dictionary};

    use super::FstDictionary;

    #[test]
    fn fst_map_contains_all_in_full_dict() {
        let dict = FstDictionary::curated();

        for word in dict.words_iter() {
            let misspelled_normalized = seq_to_normalized(word);
            let misspelled_word = misspelled_normalized.to_string();
            let misspelled_lower = misspelled_normalized.to_lower().to_string();

            dbg!(&misspelled_lower);

            assert!(!misspelled_word.is_empty());
            assert!(
                dict.word_map.contains_key(misspelled_word)
                    || dict.word_map.contains_key(misspelled_lower)
            );
        }
    }

    #[test]
    fn fst_contains_hello() {
        let dict = FstDictionary::curated();

        let word: Vec<_> = "hello".chars().collect();
        let misspelled_normalized = seq_to_normalized(&word);
        let misspelled_word: String = misspelled_normalized.to_string();
        let misspelled_lower: String = misspelled_normalized.to_lower().to_string();

        assert!(dict.contains_word(&misspelled_normalized));
        assert!(
            dict.word_map.contains_key(misspelled_lower)
                || dict.word_map.contains_key(misspelled_word)
        );
    }

    #[test]
    fn fuzzy_result_sorted_by_edit_distance() {
        let dict = FstDictionary::curated();

        let results = dict.fuzzy_match_str("hello", 3, 100);
        let is_sorted_by_dist = results
            .iter()
            .map(|fm| fm.edit_distance)
            .tuple_windows()
            .all(|(a, b)| a <= b);

        assert!(is_sorted_by_dist)
    }

    #[test]
    fn curated_contains_no_duplicates() {
        let dict = FstDictionary::curated();

        assert!(dict.words.iter().map(|(word, _)| word).all_unique());
    }
}
