//! Portuguese dictionary support.

use std::sync::{Arc, LazyLock};

use crate::spell::FstDictionary;
use crate::spell::embedded_dictionary::fst_dictionary_from_gzip_bytes;

static PORTUGUESE_DICT: LazyLock<Arc<FstDictionary>> = LazyLock::new(|| {
    Arc::new(fst_dictionary_from_gzip_bytes(
        include_bytes!("../dictionary-portuguese.dict.gz"),
        &[],
    ))
});

/// Returns a shared reference to the Portuguese FstDictionary.
pub fn portuguese_dictionary() -> Arc<FstDictionary> {
    (*PORTUGUESE_DICT).clone()
}

pub fn curated_portuguese_dictionary() -> Arc<FstDictionary> {
    portuguese_dictionary()
}
