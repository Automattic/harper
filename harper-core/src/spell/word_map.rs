use std::{borrow::Cow, sync::LazyLock};

use hashbrown::{DefaultHashBuilder, HashMap};
use indexmap::IndexMap;
use itertools::Itertools;

use crate::{
    CharString, CharStringExt, DictWordMetadata,
    edit_distance::edit_distance_min_alloc,
    spell::{
        Dictionary, FuzzyMatchResult, WordIdPair,
        dictionary::{ANNOTATIONS_STR, CURATED_DICT_STR},
        rune::{self, AttributeList, parse_word_list},
        word_id::{CanonicalWordId, CaseFoldedWordId},
    },
};

/// The underlying data structure for the `MutableDictionary`.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct WordMap {
    /// Underlying container for the entries in the word map.
    canonical: IndexMap<CanonicalWordId, WordMapEntry, DefaultHashBuilder>,
    /// A map containing indices into `canonical` for a specific `CaseFoldedWordId`. This is used for
    /// case-folded lookups in the word map.
    case_folded: HashMap<CaseFoldedWordId, Vec<usize>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct WordMapEntry {
    pub metadata: DictWordMetadata,
    pub canonical_spelling: CharString,
}

impl WordMap {
    pub fn curated() -> &'static WordMap {
        /// A word map containing entries from the curated dictionary.
        static CURATED: LazyLock<WordMap> =
            LazyLock::new(|| WordMap::from_rune_files(CURATED_DICT_STR, ANNOTATIONS_STR).unwrap());

        &CURATED
    }

    /// Does the word map contain a word with the given [`CanonicalWordId`]?
    pub fn contains_canonical(&self, id: CanonicalWordId) -> bool {
        self.get_canonical(id).is_some()
    }

    /// Does the word map contain any words with the given [`CaseFoldedWordId`]?
    pub fn contains_case_folded(&self, id: CaseFoldedWordId) -> bool {
        !self.get_canonical_indices_from_case_folded(id).is_empty()
    }

    /// Get an entry from the word map using a word identifier.
    pub fn get_canonical(&self, id: CanonicalWordId) -> Option<&WordMapEntry> {
        self.canonical.get(&id)
    }

    /// Get the entries corresponding to the provided [`CaseFoldedWordId`].
    pub fn get_case_folded(
        &self,
        id: CaseFoldedWordId,
    ) -> impl ExactSizeIterator<Item = &WordMapEntry> {
        self.get_canonical_indices_from_case_folded(id)
            .iter()
            .map(|canonical_index| self.get_by_canonical_index(*canonical_index).unwrap())
    }

    /// Convenience wrapper for [`Self::get_case_folded`].
    ///
    /// This recalculates the word ID every time. If you're going to be querying the same word
    /// multiple times, consider storing the word ID and using [`Self::get_case_folded`] instead.
    pub fn get_case_folded_chars(
        &self,
        word: &[char],
    ) -> impl ExactSizeIterator<Item = &WordMapEntry> {
        self.get_case_folded(CaseFoldedWordId::from_word_chars(word))
    }

    /// Borrow a word's metadata mutably
    pub fn get_metadata_mut_canonical(
        &mut self,
        id: CanonicalWordId,
    ) -> Option<&mut DictWordMetadata> {
        self.canonical.get_mut(&id).map(|v| &mut v.metadata)
    }

    /// Insert an entry into the word map.
    ///
    /// This will merge the new metadata with the existing if an entry with an identical
    /// [`CanonicalWordId`] is already contained in the word map.
    pub fn insert(&mut self, entry: WordMapEntry) {
        let word_ids = WordIdPair::from_word_chars(&entry.canonical_spelling);

        if let Some(existing_entry) = self.canonical.get_mut(&word_ids.canonical()) {
            // An existing word with the same canonical ID exists; update its entry.
            existing_entry.metadata.append(&entry.metadata);
        } else {
            // An existing word with the same canonical ID does NOT exist; insert it.
            let (canonical_idx, _) = self.canonical.insert_full(word_ids.canonical(), entry);
            let case_folded_id = word_ids.case_folded();
            if let Some(existing_case_folded_entry) = self.case_folded.get_mut(&case_folded_id) {
                // `case_folded` already has a canonical ID list for this word; append to it, if
                // the same entry does not already exist.
                if !existing_case_folded_entry.contains(&canonical_idx) {
                    existing_case_folded_entry.push(canonical_idx);
                }
            } else {
                // `case_folded` does NOT have a canonical ID list for this word; initialize one.
                self.case_folded.insert(case_folded_id, vec![canonical_idx]);
            }
        }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `WordMap`. The collection may reserve more space to avoid
    /// frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        self.canonical.reserve(additional);
    }

    /// Iterate through the canonical spellings of the words in the map.
    pub fn iter(&self) -> impl Iterator<Item = &WordMapEntry> {
        self.canonical.values()
    }

    /// The number of words in the word map.
    pub fn len(&self) -> usize {
        self.canonical.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            canonical: IndexMap::with_capacity_and_hasher(capacity, DefaultHashBuilder::default()),
            case_folded: HashMap::new(),
        }
    }

    pub fn from_rune_files(word_list: &str, attr_list: &str) -> Result<Self, rune::Error> {
        let word_list = parse_word_list(word_list)?;
        let attr_list = AttributeList::parse(attr_list)?;

        let mut word_map = WordMap::default();

        attr_list.expand_annotated_words(word_list, &mut word_map);

        Ok(word_map)
    }

    /// Get a [`WordMapEntry`] by its canonical ID.
    fn get_by_canonical_index(&self, index: usize) -> Option<&WordMapEntry> {
        self.canonical
            .get_index(index)
            .map(|(_, word_map_entry)| word_map_entry)
    }

    /// Get indices into [`Self::canonical`] using the provided [`CaseFoldedWordId`].
    fn get_canonical_indices_from_case_folded(&self, id: CaseFoldedWordId) -> &[usize] {
        self.case_folded
            .get(&id)
            .map_or(&[], |canonical_indices| canonical_indices)
    }
}

impl IntoIterator for WordMap {
    type Item = WordMapEntry;
    type IntoIter = indexmap::map::IntoValues<CanonicalWordId, WordMapEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.canonical.into_values()
    }
}

impl Dictionary for WordMap {
    fn get_word(&self, word: &[char]) -> Vec<&WordMapEntry> {
        self.get_case_folded(CaseFoldedWordId::from_word_chars(word))
            .collect()
    }

    fn get_word_exact(&self, word: &[char]) -> Option<&WordMapEntry> {
        self.get_canonical(CanonicalWordId::from_word_chars(word))
    }

    fn contains_word(&self, word: &[char]) -> bool {
        self.contains_case_folded(CaseFoldedWordId::from_word_chars(word))
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
                metadata: Cow::Borrowed(&self.get_word_exact(word).unwrap().metadata),
            })
            .collect()
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        Box::new(self.iter().map(|v| v.canonical_spelling.as_slice()))
    }

    fn word_count(&self) -> usize {
        self.len()
    }

    fn contains_exact_word(&self, word: &[char]) -> bool {
        self.contains_canonical(CanonicalWordId::from_word_chars(word))
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

    fn get_word_metadata_combined(&self, word: &[char]) -> Option<Cow<'_, DictWordMetadata>> {
        let mut found_words = self.get_case_folded_chars(word);

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
}
