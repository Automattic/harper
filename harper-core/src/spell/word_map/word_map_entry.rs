use crate::{CharString, DictWordMetadata, spell::WordIdPair};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct WordMapEntry {
    pub metadata: DictWordMetadata,
    pub canonical_spelling: CharString,
    pub(crate) word_ids: WordIdPair,
}

impl WordMapEntry {
    /// Construct a new word map entry with default metadata.
    pub fn new(canonical_spelling: CharString) -> Self {
        let word_ids = WordIdPair::from_word_chars(&canonical_spelling);

        Self {
            metadata: Default::default(),
            canonical_spelling,
            word_ids,
        }
    }

    /// Construct a new word map entry with default metadata.
    pub fn new_str(canonical_spelling: &str) -> Self {
        Self::new(canonical_spelling.chars().collect())
    }

    pub fn with_md(mut self, metadata: DictWordMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}
