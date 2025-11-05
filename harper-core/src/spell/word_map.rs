use hashbrown::{HashMap, hash_map::IntoValues};

use crate::{CharString, DictWordMetadata};

use super::WordId;

/// The underlying data structure for the `MutableDictionary`.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct WordMap {
    inner: HashMap<WordId, WordMapEntry>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct WordMapEntry {
    pub metadata: DictWordMetadata,
    pub canonical_spelling: CharString,
}

impl WordMap {
    /// Get an entry from the word map using raw chars.
    pub fn get_with_str(&self, string: &str) -> Option<&WordMapEntry> {
        let chars: CharString = string.chars().collect();
        let id = WordId::from_word_chars(chars);

        self.get(&id)
    }

    pub fn contains_str(&self, string: &str) -> bool {
        self.get_with_str(string).is_some()
    }

    pub fn contains_chars(&self, chars: impl AsRef<[char]>) -> bool {
        self.get_with_chars(chars).is_some()
    }

    pub fn contains(&self, id: &WordId) -> bool {
        self.get(id).is_some()
    }

    /// Get an entry from the word map using raw chars.
    pub fn get_with_chars(&self, chars: impl AsRef<[char]>) -> Option<&WordMapEntry> {
        let id = WordId::from_word_chars(chars);

        self.get(&id)
    }

    /// Get an entry from the word map using a word identifier.
    pub fn get(&self, id: &WordId) -> Option<&WordMapEntry> {
        self.inner.get(id)
    }

    /// Borrow a word's metadata mutably
    pub fn get_metadata_mut_chars(
        &mut self,
        chars: impl AsRef<[char]>,
    ) -> Option<&mut DictWordMetadata> {
        let id = WordId::from_word_chars(chars);

        self.get_metadata_mut(&id)
    }

    /// Borrow a word's metadata mutably
    pub fn get_metadata_mut(&mut self, id: &WordId) -> Option<&mut DictWordMetadata> {
        self.inner.get_mut(id).map(|v| &mut v.metadata)
    }

    pub fn insert(&mut self, entry: WordMapEntry) {
        let id = WordId::from_word_chars(&entry.canonical_spelling);

        self.inner.insert(id, entry);
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `WordMap`. The collection may reserve more space to avoid
    /// frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Iterate through the canonical spellings of the words in the map.
    pub fn iter(&self) -> impl Iterator<Item = &WordMapEntry> {
        self.inner.values()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }
}

impl IntoIterator for WordMap {
    type Item = WordMapEntry;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_values()
    }

    type IntoIter = IntoValues<WordId, WordMapEntry>;
}
