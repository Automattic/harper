use std::iter::Extend;
use std::slice::Iter;

use serde::{Deserialize, Serialize};

use crate::spell::CanonicalWordId;

/// A container for storing word IDs that a word is considered to be derived from.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Hash)]
pub struct DerivedFrom {
    inner: Vec<CanonicalWordId>,
}

impl DerivedFrom {
    /// Insert another word ID, if it's not already contained in the list.
    ///
    /// If it is already contained in the list, it's quietly ignored.
    pub fn insert(&mut self, id: CanonicalWordId) {
        if !self.contains(id) {
            self.inner.push(id);
        }
    }

    /// Is the list empty? In other words, Does this word have no known words it's derived from?
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Is this word derived from the word represented by `id`?
    pub fn contains(&self, id: CanonicalWordId) -> bool {
        self.inner.contains(&id)
    }

    /// Create a new `DerivedFrom` containing a single initial word ID.
    pub fn from_canonical_word_id(word_id: CanonicalWordId) -> Self {
        Self {
            inner: vec![word_id],
        }
    }

    /// Get an iterator of the contained [`CanonicalWordId`].
    pub fn iter(&self) -> Iter<'_, CanonicalWordId> {
        self.inner.iter()
    }
}

impl Extend<CanonicalWordId> for DerivedFrom {
    fn extend<T: IntoIterator<Item = CanonicalWordId>>(&mut self, iter: T) {
        // Extend additional word ID's, as long as they don't already exist.
        // This is intended to emulate the behavior of a `HashSet`.
        iter.into_iter().for_each(|canonical_word_id| {
            self.insert(canonical_word_id);
        });
    }
}

impl<'a> Extend<&'a CanonicalWordId> for DerivedFrom {
    fn extend<T: IntoIterator<Item = &'a CanonicalWordId>>(&mut self, iter: T) {
        // Extend additional word ID's, as long as they don't already exist.
        // This is intended to emulate the behavior of a `HashSet`.
        iter.into_iter().copied().for_each(|canonical_word_id| {
            self.insert(canonical_word_id);
        });
    }
}

impl IntoIterator for DerivedFrom {
    type Item = CanonicalWordId;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
