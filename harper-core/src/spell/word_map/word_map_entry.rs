use crate::{CharString, DictWordMetadata};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct WordMapEntry {
    pub metadata: DictWordMetadata,
    pub canonical_spelling: CharString,
}
