use std::{
    borrow::Cow,
    cell::RefCell,
    sync::{Arc, LazyLock},
};

use fst::{IntoStreamer, Map as FstMap, Streamer, map::StreamWithState};
use hashbrown::HashMap;
use levenshtein_automata::{DFA, LevenshteinAutomatonBuilder};

use super::{Dictionary, FuzzyMatchResult, MutableDictionary, WordId};
use crate::{CharString, CharStringExt, DictWordMetadata};

/// An immutable dictionary allowing for very fast spellchecking.
///
/// For dictionaries with changing contents, such as user and file dictionaries, prefer
/// [`MutableDictionary`].
pub struct FstDictionary {
    /// Underlying [`super::MutableDictionary`] used for everything except fuzzy finding
    mutable_dict: Arc<MutableDictionary>,
    /// Used for fuzzy-finding the index of words or metadata
    word_map: FstMap<Vec<u8>>,
    /// Used for fuzzy-finding the index of words or metadata
    words: Vec<(CharString, DictWordMetadata)>,
}

const EXPECTED_DISTANCE: u8 = 3;
const TRANSPOSITION_COST_ONE: bool = true;

static DICT: LazyLock<Arc<FstDictionary>> =
    LazyLock::new(|| Arc::new((*MutableDictionary::curated()).clone().into()));

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
        self.mutable_dict == other.mutable_dict
    }
}

impl FstDictionary {
    /// Create a dictionary from the curated dictionary included
    /// in the Harper binary.
    pub fn curated() -> Arc<Self> {
        (*DICT).clone()
    }

    /// Construct a new [`FstDictionary`] using a wordlist as a source.
    /// This can be expensive, so only use this if fast fuzzy searches are worth it.
    pub fn new(mut words: Vec<(CharString, DictWordMetadata)>) -> Self {
        words.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        words.dedup_by(|(a, _), (b, _)| a == b);

        let mut builder = fst::MapBuilder::memory();
        for (index, (word, _)) in words.iter().enumerate() {
            let word = word.iter().collect::<String>();
            builder
                .insert(word, index as u64)
                .expect("Insertion not in lexicographical order!");
        }

        let mut mutable_dict = MutableDictionary::new();
        mutable_dict.extend_words(words.iter().cloned());

        let fst_bytes = builder.into_inner().unwrap();
        let word_map = FstMap::new(fst_bytes).expect("Unable to build FST map.");

        FstDictionary {
            mutable_dict: Arc::new(mutable_dict),
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

/// Merges index-distance pairs, keeping the smallest distance for each word.
fn merge_best_distances(
    best_distances: &mut HashMap<u64, u8>,
    distances: impl IntoIterator<Item = (u64, u8)>,
) {
    for (idx, dist) in distances {
        best_distances
            .entry(idx)
            .and_modify(|existing| *existing = (*existing).min(dist))
            .or_insert(dist);
    }
}

impl Dictionary for FstDictionary {
    fn contains_word(&self, word: &[char]) -> bool {
        self.mutable_dict.contains_word(word)
    }

    fn contains_word_str(&self, word: &str) -> bool {
        self.mutable_dict.contains_word_str(word)
    }

    fn get_word_metadata(&self, word: &[char]) -> Option<Cow<'_, DictWordMetadata>> {
        self.mutable_dict.get_word_metadata(word)
    }

    fn get_word_metadata_str(&self, word: &str) -> Option<Cow<'_, DictWordMetadata>> {
        self.mutable_dict.get_word_metadata_str(word)
    }

    fn fuzzy_match(
        &'_ self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>> {
        let misspelled_word_charslice = word.normalized();
        let misspelled_word_string = misspelled_word_charslice.to_string();
        let misspelled_lower = misspelled_word_string.to_lowercase();
        let is_already_lower = misspelled_lower == misspelled_word_string;

        // Actual FST search
        let dfa = build_dfa(max_distance, &misspelled_word_string);
        let mut word_indexes_stream = self.word_map.search_with_state(&dfa).into_stream();
        let upper_dists = stream_distances_vec(&mut word_indexes_stream, &dfa);

        // Merge the two results, keeping the smallest distance when both DFAs match.
        // The uppercase and lowercase searches can return different result counts, so
        // we can't simply zip the vectors without losing matches.
        let mut best_distances = HashMap::<u64, u8>::new();

        merge_best_distances(&mut best_distances, upper_dists);

        // Only build the lowercase DFA when the query is not already lowercase.
        if !is_already_lower {
            let dfa_lowercase = build_dfa(max_distance, &misspelled_lower);
            let mut word_indexes_lowercase_stream = self
                .word_map
                .search_with_state(&dfa_lowercase)
                .into_stream();
            let lower_dists =
                stream_distances_vec(&mut word_indexes_lowercase_stream, &dfa_lowercase);

            merge_best_distances(&mut best_distances, lower_dists);
        }

        let mut merged = Vec::with_capacity(best_distances.len());
        for (index, edit_distance) in best_distances {
            let (word, metadata) = &self.words[index as usize];
            merged.push(FuzzyMatchResult {
                word,
                edit_distance,
                metadata: Cow::Borrowed(metadata),
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

    fn fuzzy_match_str(
        &'_ self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>> {
        self.fuzzy_match(
            word.chars().collect::<Vec<_>>().as_slice(),
            max_distance,
            max_results,
        )
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        self.mutable_dict.words_iter()
    }

    fn word_count(&self) -> usize {
        self.mutable_dict.word_count()
    }

    fn contains_exact_word(&self, word: &[char]) -> bool {
        self.mutable_dict.contains_exact_word(word)
    }

    fn contains_exact_word_str(&self, word: &str) -> bool {
        self.mutable_dict.contains_exact_word_str(word)
    }

    fn get_correct_capitalization_of(&self, word: &[char]) -> Option<&'_ [char]> {
        self.mutable_dict.get_correct_capitalization_of(word)
    }

    fn get_word_from_id(&self, id: &WordId) -> Option<&[char]> {
        self.mutable_dict.get_word_from_id(id)
    }

    fn find_words_with_prefix(&self, prefix: &[char]) -> Vec<Cow<'_, [char]>> {
        self.mutable_dict.find_words_with_prefix(prefix)
    }

    fn find_words_with_common_prefix(&self, word: &[char]) -> Vec<Cow<'_, [char]>> {
        self.mutable_dict.find_words_with_common_prefix(word)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::CharStringExt;
    use crate::DictWordMetadata;
    use crate::spell::{Dictionary, MutableDictionary, WordId};

    use super::FstDictionary;

    fn test_dictionaries(words: &[&str]) -> (MutableDictionary, FstDictionary) {
        let mut mutable = MutableDictionary::new();

        for word in words {
            mutable.append_word_str(word, DictWordMetadata::default());
        }

        let fst = FstDictionary::from(mutable.clone());

        (mutable, fst)
    }

    fn fuzzy_matches<D: Dictionary + ?Sized>(
        dict: &D,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<(String, u8)> {
        let mut matches = dict
            .fuzzy_match_str(word, max_distance, max_results)
            .into_iter()
            .map(|result| (result.word.iter().collect::<String>(), result.edit_distance))
            .collect_vec();

        matches.sort_unstable_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        matches
    }

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
    fn fst_map_contains_all_in_mutable_dict() {
        let dict = FstDictionary::curated();

        for word in dict.words_iter() {
            let misspelled_normalized = word.normalized();
            let misspelled_word = misspelled_normalized.to_string();
            let misspelled_lower = misspelled_normalized.to_lower().to_string();

            dbg!(&misspelled_lower);

            assert!(!misspelled_word.is_empty());
            assert!(dict.word_map.contains_key(misspelled_word));
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
            dict.word_map.contains_key(misspelled_lower)
                || dict.word_map.contains_key(misspelled_word)
        );
    }

    #[test]
    fn on_is_not_nominal() {
        let dict = FstDictionary::curated();

        assert!(!dict.get_word_metadata_str("on").unwrap().is_nominal());
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

    #[test]
    fn contractions_not_derived() {
        let dict = FstDictionary::curated();

        let contractions = ["there's", "we're", "here's"];

        for contraction in contractions {
            dbg!(contraction);
            assert!(
                dict.get_word_metadata_str(contraction)
                    .unwrap()
                    .derived_from
                    .is_none()
            )
        }
    }

    #[test]
    fn plural_llamas_derived_from_llama() {
        let dict = FstDictionary::curated();

        assert_eq!(
            dict.get_word_metadata_str("llamas")
                .unwrap()
                .derived_from
                .unwrap(),
            WordId::from_word_str("llama")
        )
    }

    #[test]
    fn plural_cats_derived_from_cat() {
        let dict = FstDictionary::curated();

        assert_eq!(
            dict.get_word_metadata_str("cats")
                .unwrap()
                .derived_from
                .unwrap(),
            WordId::from_word_str("cat")
        );
    }

    #[test]
    fn unhappy_derived_from_happy() {
        let dict = FstDictionary::curated();

        assert_eq!(
            dict.get_word_metadata_str("unhappy")
                .unwrap()
                .derived_from
                .unwrap(),
            WordId::from_word_str("happy")
        );
    }

    #[test]
    fn quickly_derived_from_quick() {
        let dict = FstDictionary::curated();

        assert_eq!(
            dict.get_word_metadata_str("quickly")
                .unwrap()
                .derived_from
                .unwrap(),
            WordId::from_word_str("quick")
        );
    }

    #[test]
    fn lowercase_fuzzy_match_matches_mutable_dictionary() {
        let (mutable, fst) =
            test_dictionaries(&["spelling", "spilling", "selling", "smelling", "shelling"]);

        let mutable_results = fuzzy_matches(&mutable, "speling", 3, 10);
        let fst_results = fuzzy_matches(&fst, "speling", 3, 10);

        assert_eq!(fst_results, mutable_results);
        assert_eq!(fst_results.first(), Some(&(String::from("spelling"), 1)));
    }

    #[test]
    fn capitalized_fuzzy_match_matches_mutable_dictionary() {
        let (mutable, fst) =
            test_dictionaries(&["spelling", "spilling", "selling", "smelling", "shelling"]);

        let mutable_results = fuzzy_matches(&mutable, "Speling", 3, 10);
        let fst_results = fuzzy_matches(&fst, "Speling", 3, 10);

        assert_eq!(fst_results, mutable_results);
        assert_eq!(fst_results.first(), Some(&(String::from("spelling"), 1)));
    }

    #[test]
    fn uppercase_fuzzy_match_matches_mutable_dictionary() {
        let (mutable, fst) =
            test_dictionaries(&["spelling", "spilling", "selling", "smelling", "shelling"]);

        let mutable_results = fuzzy_matches(&mutable, "SPELING", 3, 10);
        let fst_results = fuzzy_matches(&fst, "SPELING", 3, 10);

        assert_eq!(fst_results, mutable_results);
        assert_eq!(fst_results.first(), Some(&(String::from("spelling"), 1)));
    }

    #[test]
    fn query_casing_produces_the_same_fuzzy_matches() {
        let (_, fst) =
            test_dictionaries(&["spelling", "spilling", "selling", "smelling", "shelling"]);

        let lowercase = fuzzy_matches(&fst, "speling", 3, 10);
        let capitalized = fuzzy_matches(&fst, "Speling", 3, 10);
        let uppercase = fuzzy_matches(&fst, "SPELING", 3, 10);

        assert_eq!(lowercase, capitalized);
        assert_eq!(lowercase, uppercase);
    }
}
