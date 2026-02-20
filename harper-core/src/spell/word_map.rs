use std::sync::LazyLock;

use hashbrown::{DefaultHashBuilder, HashMap};
use indexmap::IndexMap;

use crate::{
    CharString, DictWordMetadata,
    spell::{
        WordIdPair,
        dictionary::{ANNOTATIONS_STR, CURATED_DICT_STR},
        rune::{self, AttributeList, parse_word_list},
        word_id::{CanonicalWordId, CaseFoldedWordId},
    },
};

/// A word map containing entries from the curated dictionary.
pub(crate) static CURATED: LazyLock<WordMap> =
    LazyLock::new(|| WordMap::from_rune_files(CURATED_DICT_STR, ANNOTATIONS_STR).unwrap());

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

    fn into_iter(self) -> Self::IntoIter {
        self.canonical.into_values()
    }

    type IntoIter = indexmap::map::IntoValues<CanonicalWordId, WordMapEntry>;
}
