use std::sync::OnceLock;

use hashbrown::HashMap;
use indexmap::IndexSet;

static COMPRESSED_THESAURUS: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/compressed-thesaurus.zst"));
static RAW_WORD_FREQUENCY_TEXT: &str = include_str!("../word-freq.txt");

/// Gets a read-only reference to the thesaurus.
pub fn thesaurus() -> &'static Thesaurus {
    static THESAURUS: OnceLock<Thesaurus> = OnceLock::new();
    THESAURUS.get_or_init(Thesaurus::new)
}

/// A list of words numbered by frequency of use. The most common word will have a number of 0, and
/// rarer words count up from there.
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
    /// Contains the words in the thesaurus and their corresponding synonyms, both as indices into
    /// [`Self::deduped_word_set`].
    entries: HashMap<usize, Vec<usize>>,
    /// Contains (and holds ownership of) all words that occur in the thesaurus, deduped.
    deduped_word_set: IndexSet<String>,
}
impl Thesaurus {
    fn new() -> Thesaurus {
        let mut entries = HashMap::new();
        let mut deduped_word_set = IndexSet::<String>::new();

        let raw_thesaurus_text = zstd::stream::decode_all(COMPRESSED_THESAURUS)
            .expect("Compressed thesaurus is a valid ZSTD file")
            .into_boxed_slice();
        let raw_thesaurus_text =
            str::from_utf8(&raw_thesaurus_text).expect("Thesaurus content is valid UTF-8");

        for line in raw_thesaurus_text.lines() {
            let mut words = line.split(',');
            let Some(entry_word) = words.next() else {
                // Skip empty lines in thesaurus.
                continue;
            };
            let word_idx = deduped_word_set.get_or_insert_word(entry_word);
            let synonym_indices = words.map(|word| deduped_word_set.get_or_insert_word(word));
            entries
                .try_insert(word_idx, synonym_indices.collect())
                .expect("Only one entry per word in thesaurus");
        }

        Self {
            entries,
            deduped_word_set,
        }
    }

    /// Retrieves a list of synonyms for a given word.
    pub fn get_synonyms(&self, word: &str) -> Option<Vec<&str>> {
        Some(
            self.entries
                .get(&self.deduped_word_set.get_index_of(word)?)?
                .iter()
                .map(|word_idx| -> &str {
                    self.deduped_word_set
                        .get_index(*word_idx)
                        .expect("Deduped word set contains all words in thesaurus")
                })
                .collect(),
        )
    }

    /// Retrieves a list of synonyms, sorted by the frequency of their use.
    pub fn get_synonyms_freq_sorted(&self, word: &str) -> Option<Vec<&str>> {
        let mut syns = self.get_synonyms(word)?;
        syns.sort_unstable_by_key(|syn| {
            word_freq_map()
                .get(&syn.to_ascii_lowercase())
                .unwrap_or(&u32::MAX)
        });
        Some(syns)
    }
}

trait DedupedWordSetExt {
    /// Gets or insert the provided word.
    ///
    /// Returns the index of the word.
    fn get_or_insert_word(&mut self, word: &str) -> usize;
}
impl DedupedWordSetExt for IndexSet<String> {
    fn get_or_insert_word(&mut self, word: &str) -> usize {
        if let Some(idx) = self.get_index_of(word) {
            idx
        } else {
            // Avoid cloning unless we're inserting a new word.
            self.insert_full(word.to_owned()).0
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn great_is_synonym_of_large() {
        assert!(
            super::thesaurus()
                .get_synonyms("large")
                .is_some_and(|syns| syns.contains(&"great"))
        );
    }
}
