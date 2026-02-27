use std::{borrow::Cow, ops::Index, sync::LazyLock};

use hashbrown::{DefaultHashBuilder, HashMap};
use indexmap::IndexMap;
use itertools::Itertools;

use crate::{
    CharStringExt, DictWordMetadata,
    edit_distance::edit_distance_min_alloc,
    spell::{
        CommonDictFuncs, Dictionary, FstDictionary, FuzzyMatchResult,
        dictionary::{ANNOTATIONS_STR, CURATED_DICT_STR},
        rune::{self, AttributeList, parse_word_list},
        word_id::{CanonicalWordId, CaseFoldedWordId},
    },
};

pub mod word_map_entry;
pub use word_map_entry::WordMapEntry;

/// Type used to store the canonical entries in the word map.
///
/// This currently uses an [`IndexMap`], so a word can be indexed either by its canonical word ID
/// (which is the same across all word maps), or directly via a `usize` index (specific to this
/// word map only).
///
/// The latter property is used to allow [`WordMap::case_folded`] to indirectly reference entries
/// in [`WordMap::canonical`].
type CanonicalStorage = IndexMap<CanonicalWordId, WordMapEntry, DefaultHashBuilder>;

/// The most basic dictionary. Stores a list of words and their metadata, while allowing for
/// efficient case-folded lookups.
///
/// This is the underlying data structure for all other dictionaries (at the time of writing).
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct WordMap {
    /// Underlying container for the entries in the word map.
    canonical: CanonicalStorage,
    /// A map containing indices into `canonical` for a specific `CaseFoldedWordId`. This is used for
    /// case-folded lookups in the word map.
    case_folded: HashMap<CaseFoldedWordId, Vec<usize>>,
}

impl WordMap {
    /// Create an empty word map.
    pub fn new() -> Self {
        Default::default()
    }

    /// A word map containing entries from the curated dictionary.
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
    ) -> impl ExactSizeIterator<Item = &WordMapEntry> + use<'_> {
        self.get_case_folded(CaseFoldedWordId::from_word_chars(word).0)
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
    ///
    /// If you are appending many words, consider using [`Self::extend`] instead.
    pub fn insert(&mut self, entry: WordMapEntry) {
        let (canonical_id, case_folded_id) =
            (entry.word_ids.canonical(), entry.word_ids.case_folded());

        if let Some(existing_entry) = self.canonical.get_mut(&canonical_id) {
            // An existing word with the same canonical ID exists; update its entry.
            existing_entry.metadata.append(&entry.metadata);
        } else {
            // An existing word with the same canonical ID does NOT exist; insert it.
            let (canonical_idx, _) = self.canonical.insert_full(canonical_id, entry);
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
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &WordMapEntry> {
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

    /// Create an [`FstDictionary`] from this word map.
    pub fn to_fst(self) -> FstDictionary {
        FstDictionary::new(self)
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

impl Extend<WordMapEntry> for WordMap {
    fn extend<T: IntoIterator<Item = WordMapEntry>>(&mut self, iter: T) {
        // [Copied from HashBrown::map::HashMap::extend]
        // Keys may be already present or show multiple times in the iterator.
        // Reserve the entire hint lower bound if the map is empty.
        // Otherwise reserve half the hint (rounded up), so the map
        // will only resize twice in the worst case.
        let iter = iter.into_iter();
        let reserve = if self.is_empty() {
            iter.size_hint().0
        } else {
            iter.size_hint().0.div_ceil(2)
        };
        self.reserve(reserve);
        iter.for_each(move |wme| {
            self.insert(wme);
        });
    }
}

impl Index<usize> for WordMap {
    type Output = <CanonicalStorage as Index<usize>>::Output;

    fn index(&self, index: usize) -> &Self::Output {
        &self.canonical[index]
    }
}

impl IntoIterator for WordMap {
    type Item = WordMapEntry;
    type IntoIter = indexmap::map::IntoValues<CanonicalWordId, WordMapEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.canonical.into_values()
    }
}

impl FromIterator<WordMapEntry> for WordMap {
    fn from_iter<T: IntoIterator<Item = WordMapEntry>>(iter: T) -> Self {
        let mut out = Self::new();
        out.extend(iter);
        out
    }
}

impl Dictionary for WordMap {
    fn get_word_map(&self) -> &WordMap {
        self
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
                metadata: Cow::Borrowed(&self.get_word_metadata_exact(word).unwrap()),
            })
            .collect()
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
