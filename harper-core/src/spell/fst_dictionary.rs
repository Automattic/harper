use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::LazyLock;

use fst::{IntoStreamer, Map as FstMap, Streamer, map::StreamWithState};
use hashbrown::HashMap;
use levenshtein_automata::{DFA, LevenshteinAutomatonBuilder};

use super::{Dictionary, FuzzyMatchResult, WordMap, WordMapEntry};
use crate::CharStringExt;

/// An immutable dictionary allowing for very fast spellchecking.
///
/// For dictionaries with changing contents, such as user and file dictionaries, prefer
/// [`WordMap`].
pub struct FstDictionary {
    /// Underlying [`super::WordMap`] used for everything except fuzzy finding
    word_map: WordMap,
    /// Used for fuzzy-finding the index of words or metadata
    fst_map: FstMap<Vec<u8>>,
}

const EXPECTED_DISTANCE: u8 = 3;
const TRANSPOSITION_COST_ONE: bool = true;

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
        self.word_map == other.word_map
    }
}

impl FstDictionary {
    /// Create a dictionary from the curated dictionary included
    /// in the Harper binary.
    pub fn curated() -> &'static FstDictionary {
        static DICT: LazyLock<FstDictionary> =
            LazyLock::new(|| WordMap::curated().clone().to_fst());

        &DICT
    }

    /// Construct a new [`FstDictionary`] using a wordlist as a source.
    /// This can be expensive, so only use this if fast fuzzy searches are worth it.
    pub fn new(mut words: Vec<WordMapEntry>) -> Self {
        words.sort_unstable_by(|a, b| a.canonical_spelling.cmp(&b.canonical_spelling));
        words.dedup_by(|a, b| a.canonical_spelling == b.canonical_spelling);

        let mut builder = fst::MapBuilder::memory();
        for (index, wme) in words.iter().enumerate() {
            let word = wme.canonical_spelling.to_string();
            builder
                .insert(word, index as u64)
                .expect("Insertion not in lexicographical order!");
        }

        let word_map = WordMap::from_iter(words);

        let fst_bytes = builder.into_inner().unwrap();
        let fst_map = FstMap::new(fst_bytes).expect("Unable to build FST map.");

        FstDictionary { word_map, fst_map }
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
    fn get_word_map(&self) -> &WordMap {
        self.word_map.get_word_map()
    }

    fn fuzzy_match(
        &'_ self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>> {
        let misspelled_word_charslice = word.normalized();
        let misspelled_word_string = misspelled_word_charslice.to_string();

        // Actual FST search
        let dfa = build_dfa(max_distance, &misspelled_word_string);
        let dfa_lowercase = build_dfa(max_distance, &misspelled_word_string.to_lowercase());
        let mut word_indexes_stream = self.fst_map.search_with_state(&dfa).into_stream();
        let mut word_indexes_lowercase_stream =
            self.fst_map.search_with_state(&dfa_lowercase).into_stream();

        let upper_dists = stream_distances_vec(&mut word_indexes_stream, &dfa);
        let lower_dists = stream_distances_vec(&mut word_indexes_lowercase_stream, &dfa_lowercase);

        // Merge the two results, keeping the smallest distance when both DFAs match.
        // The uppercase and lowercase searches can return different result counts, so
        // we can't simply zip the vectors without losing matches.
        let mut merged = Vec::with_capacity(upper_dists.len().max(lower_dists.len()));
        let mut best_distances = HashMap::<u64, u8>::new();

        for (idx, dist) in upper_dists.into_iter().chain(lower_dists.into_iter()) {
            best_distances
                .entry(idx)
                .and_modify(|existing| *existing = (*existing).min(dist))
                .or_insert(dist);
        }

        for (index, edit_distance) in best_distances {
            let wme = &self.word_map[index as usize];
            merged.push(FuzzyMatchResult {
                word: &wme.canonical_spelling,
                edit_distance,
                metadata: Cow::Borrowed(&wme.metadata),
            });
        }

        // Ignore exact matches
        merged.retain(|v| v.edit_distance > 0);
        merged.sort_unstable_by(|a, b| {
            a.edit_distance
                .cmp(&b.edit_distance)
                .then_with(|| a.word.cmp(b.word))
        });
        merged.truncate(max_results);

        merged
    }

    fn find_words_with_prefix(&self, prefix: &[char]) -> Vec<Cow<'_, [char]>> {
        self.word_map.find_words_with_prefix(prefix)
    }

    fn find_words_with_common_prefix(&self, word: &[char]) -> Vec<Cow<'_, [char]>> {
        self.word_map.find_words_with_common_prefix(word)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::CharStringExt;
    use crate::spell::{CanonicalWordId, CommonDictFuncs, Dictionary};

    use super::FstDictionary;

    #[test]
    fn damerau_transposition_costs_one() {
        let lev_automata =
            levenshtein_automata::LevenshteinAutomatonBuilder::new(1, true).build_dfa("woof");
        assert_eq!(
            lev_automata.eval("wofo"),
            levenshtein_automata::Distance::Exact(1)
        );
    }

    #[test]
    fn damerau_transposition_costs_two() {
        let lev_automata =
            levenshtein_automata::LevenshteinAutomatonBuilder::new(1, false).build_dfa("woof");
        assert_eq!(
            lev_automata.eval("wofo"),
            levenshtein_automata::Distance::AtLeast(2)
        );
    }

    #[test]
    fn fst_map_contains_all_in_curated_dict() {
        let dict = FstDictionary::curated();

        for word in dict.words_iter() {
            let misspelled_normalized = word.normalized();
            let misspelled_word = misspelled_normalized.to_string();
            let misspelled_lower = misspelled_normalized.to_lower().to_string();

            dbg!(&misspelled_lower);

            assert!(!misspelled_word.is_empty());
            assert!(dict.fst_map.contains_key(misspelled_word));
        }
    }

    #[test]
    fn fst_contains_hello() {
        let dict = FstDictionary::curated();

        let word: Vec<_> = "hello".chars().collect();
        let misspelled_normalized = word.normalized();
        let misspelled_word = misspelled_normalized.to_string();
        let misspelled_lower = misspelled_normalized.to_lower().to_string();

        assert!(dict.contains_word(&misspelled_normalized));
        assert!(
            dict.fst_map.contains_key(misspelled_lower)
                || dict.fst_map.contains_key(misspelled_word)
        );
    }

    #[test]
    fn on_is_not_nominal() {
        let dict = FstDictionary::curated();

        assert!(!dict.get_word_exact_str("on").unwrap().metadata.is_nominal());
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

        assert!(
            dict.word_map
                .iter()
                .map(|wme| &wme.canonical_spelling)
                .all_unique()
        );
    }

    #[test]
    fn contractions_not_derived() {
        let dict = FstDictionary::curated();

        let contractions = ["there's", "we're", "here's"];

        for contraction in contractions {
            dbg!(contraction);
            assert!(
                dict.get_word_exact_str(contraction)
                    .unwrap()
                    .metadata
                    .derived_from
                    .is_empty()
            )
        }
    }

    #[test]
    fn plural_llamas_derived_from_llama() {
        let dict = FstDictionary::curated();

        assert!(
            dict.get_word_exact_str("llamas")
                .unwrap()
                .metadata
                .derived_from
                .contains(CanonicalWordId::from_word_str("llama"))
        )
    }

    #[test]
    fn plural_cats_derived_from_cat() {
        let dict = FstDictionary::curated();

        assert!(
            dict.get_word_exact_str("cats")
                .unwrap()
                .metadata
                .derived_from
                .contains(CanonicalWordId::from_word_str("cat"))
        );
    }

    #[test]
    fn unhappy_derived_from_happy() {
        let dict = FstDictionary::curated();

        assert!(
            dict.get_word_exact_str("unhappy")
                .unwrap()
                .metadata
                .derived_from
                .contains(CanonicalWordId::from_word_str("happy"))
        );
    }

    #[test]
    fn quickly_derived_from_quick() {
        let dict = FstDictionary::curated();

        assert!(
            dict.get_word_exact_str("quickly")
                .unwrap()
                .metadata
                .derived_from
                .contains(CanonicalWordId::from_word_str("quick"))
        );
    }
}
