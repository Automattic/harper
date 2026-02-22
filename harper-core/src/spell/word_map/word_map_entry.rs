use crate::{CharString, DictWordMetadata};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct WordMapEntry {
    pub metadata: DictWordMetadata,
    pub canonical_spelling: CharString,
}

impl WordMapEntry {
    /// Construct a new word map entry with default metadata.
    pub fn new(canonical_spelling: CharString) -> Self {
        Self {
            metadata: Default::default(),
            canonical_spelling,
        }
    }

    /// Construct a new word map entry with default metadata.
    pub fn new_str(canonical_spelling: &str) -> Self {
        Self {
            metadata: Default::default(),
            canonical_spelling: canonical_spelling.chars().collect(),
        }
    }

    pub fn with_md(mut self, metadata: DictWordMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}
