#![warn(clippy::pedantic)]

use std::ops::Range;
use std::sync::OnceLock;

use hashbrown::HashMap;
use hashbrown::HashSet;

static RAW_THESAURUS_TEXT: &str = include_str!("../thesaurus.txt");
static RAW_WORD_FREQUENCY_TEXT: &str = include_str!("../word-freq.txt");

/// Gets a read-only reference to the thesaurus.
pub fn thesaurus() -> &'static Thesaurus {
    static THESAURUS: OnceLock<Thesaurus> = OnceLock::new();
    THESAURUS.get_or_init(Thesaurus::new)
}

/// The set of all words mentioned in the thesaurus.
fn deduped_word_set() -> &'static HashSet<&'static str> {
    static DEDUPED_WORD_SET: OnceLock<HashSet<&str>> = OnceLock::new();
    DEDUPED_WORD_SET.get_or_init(|| {
        let mut deduped_word_set = HashSet::new();
        for line in RAW_THESAURUS_TEXT.lines() {
            deduped_word_set.extend(line.split(','));
        }
        deduped_word_set
    })
}

/// A list of words sorted by frequency of use, in descending order.
fn word_freq_map() -> &'static HashMap<String, u32> {
    static WORD_FREQ_LIST: OnceLock<HashMap<String, u32>> = OnceLock::new();
    WORD_FREQ_LIST.get_or_init(|| {
        RAW_WORD_FREQUENCY_TEXT
            .lines()
            .enumerate()
            .map(|(i, word)| (word.to_owned(), u32::try_from(i).unwrap()))
            .collect()
    })
}

pub struct Thesaurus {
    /// Contains the words in the thesaurus and their corresponding synonyms.
    entries: HashMap<&'static str, Range<usize>>,
    /// Holds references to words in the [`deduped_word_set()`]. Slices from this list are used by
    /// [`Self::entries`] to store the list of synonyms for each word.
    ///
    /// This is an optimization that avoids having to store each slice of synonyms as a separate
    /// heap allocation. Instead, all such would be slices are stored in one [`Vec<&'static str>`],
    /// and subslices of that collection are used instead.
    ///
    /// Note that ranges are used in place of direct slices to avoid creating a self-referential
    /// struct. The actual words can be retrieved by calling [`WordRefListCollection::get_words()`]
    /// with the range in question.
    word_ref_list_collection: WordRefListCollection,
}
impl Thesaurus {
    fn new() -> Thesaurus {
        let mut entries = HashMap::new();
        let mut word_ref_list_collection = WordRefListCollection::new();

        for line in RAW_THESAURUS_TEXT.lines() {
            let mut words = line.split(',');
            let Some(entry_word) = words.next() else {
                // Skip empty lines in thesaurus.
                continue;
            };
            let entry_word = deduped_word_set()
                .get(entry_word)
                .expect("Deduped wordset contains all words from thesaurus");
            let synonyms = words.map(|word| {
                deduped_word_set()
                    .get(&word)
                    .expect("Deduped wordset contains all words from thesaurus")
            });
            entries
                .try_insert(
                    *entry_word,
                    word_ref_list_collection.create_word_ref_list(synonyms),
                )
                .expect("Only one entry per word in thesaurus");
        }

        Self {
            entries,
            word_ref_list_collection,
        }
    }

    /// Retrieves a list of synonyms for a given word.
    pub fn get_synonyms(&self, word: &str) -> Option<&[&'static str]> {
        self.word_ref_list_collection
            .get_words(self.entries.get(word)?)
    }

    /// Retrieves a list of synonyms, sorted by the frequency of their use.
    pub fn get_synonyms_freq_sorted(&self, word: &str) -> Option<Vec<&'static str>> {
        let mut syns = self.get_synonyms(word)?.to_owned();
        syns.sort_unstable_by_key(|syn| {
            word_freq_map()
                .get(&syn.to_ascii_lowercase())
                .unwrap_or(&u32::MAX)
        });
        Some(syns)
    }
}

struct WordRefListCollection {
    aggregator: Vec<&'static str>,
}
impl WordRefListCollection {
    /// Creates an empty [`WordRefListCollection`].
    fn new() -> Self {
        Self {
            aggregator: Vec::new(),
        }
    }

    /// Creates a new word ref list and returns a range to it.
    ///
    /// The range can then be used with [`Self::get_words()`], to retrieve the list of words.
    ///
    /// Expects a reference to an entry in the [`deduped_word_set`], which itself points to a slice in
    /// the [`RAW_THESAURUS_TEXT`]. Internally, this `&&` is flattened to a `&`, which means that
    /// later accesses with [`Self::get_words()`] or similar will return references to
    /// [`RAW_THESAURUS_TEXT`] directly.
    ///
    /// This is designed to improve caching, since all occurrences of a given word will only ever
    /// point to a single instance of that word in [`RAW_THESAURUS_TEXT`].
    fn create_word_ref_list(
        &mut self,
        words: impl IntoIterator<Item = &'static &'static str>,
    ) -> Range<usize> {
        let start_idx_of_new_words = self.aggregator.len();
        self.aggregator.extend(words);
        start_idx_of_new_words..self.aggregator.len()
    }

    /// Retrieves the words from a range previously generated by [`Self::create_word_ref_list()`].
    ///
    /// Returns [`None`] on any failure.
    fn get_words(&self, range: &Range<usize>) -> Option<&[&'static str]> {
        self.aggregator.get(range.clone())
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::thesaurus::deduped_word_set;

    #[test]
    fn great_is_synonym_of_large() {
        assert!(
            super::thesaurus()
                .get_synonyms("large")
                .is_some_and(|syns| syns.contains(&"great"))
        );
    }

    #[test]
    fn all_entries_of_the_same_synonym_point_to_the_same_str() {
        let word = "any";
        assert!(
            deduped_word_set()
                .iter()
                .filter_map(|word| super::thesaurus().get_synonyms(word))
                .flatten()
                .filter(|syn| **syn == word)
                .map(|syn| syn.as_ptr())
                .all_equal()
        );
    }
}
