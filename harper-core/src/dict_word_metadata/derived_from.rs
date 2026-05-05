use std::{
    collections::{BTreeSet, btree_set},
    iter::Extend,
};

use serde::{Deserialize, Serialize};

use crate::spell::CanonicalWordId;

/// A container for storing word IDs that a word is considered to be derived from.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Hash)]
pub struct DerivedFrom {
    inner: BTreeSet<CanonicalWordId>,
}

impl DerivedFrom {
    /// Insert another word ID, if it's not already contained.
    ///
    /// If it is already contained, it's quietly ignored.
    pub fn insert(&mut self, id: CanonicalWordId) {
        self.inner.insert(id);
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
            inner: BTreeSet::from([word_id]),
        }
    }

    /// Get an iterator of the contained [`CanonicalWordId`].
    pub fn iter(&self) -> btree_set::Iter<'_, CanonicalWordId> {
        self.inner.iter()
    }
}

impl Extend<CanonicalWordId> for DerivedFrom {
    fn extend<T: IntoIterator<Item = CanonicalWordId>>(&mut self, iter: T) {
        self.inner.extend(iter);
    }
}

impl<'a> Extend<&'a CanonicalWordId> for DerivedFrom {
    fn extend<T: IntoIterator<Item = &'a CanonicalWordId>>(&mut self, iter: T) {
        self.inner.extend(iter);
    }
}

impl IntoIterator for DerivedFrom {
    type Item = CanonicalWordId;

    type IntoIter = btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
