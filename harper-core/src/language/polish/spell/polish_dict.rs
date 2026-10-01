//! Polish dictionary support.
//!
//! This module provides the Polish dictionary implementation.
//! Uses Rune format dictionaries with annotations for comprehensive coverage.

use crate::spell::{FstDictionary, MutableDictionary};
use std::sync::{Arc, LazyLock};

fn load_polish_annotated_dict() -> Arc<MutableDictionary> {
    MutableDictionary::from_rune_files(
        include_str!("../dictionary.dict"),
        include_str!("../annotations.json"),
    )
    .map(Arc::new)
    .unwrap_or_else(|e| panic!("Failed to load Polish annotated dictionary: {}", e))
}

// New annotated dictionary using Rune format
static POLISH_ANNOTATED_DICT: LazyLock<Arc<MutableDictionary>> =
    LazyLock::new(load_polish_annotated_dict);

/// Returns a shared reference to the original Polish FstDictionary.
///
/// The dictionary is loaded and built once on first access, then cached for the
/// lifetime of the process. This provides fuzzy matching, prefix search, and
/// all other `Dictionary` trait capabilities.
pub fn polish_dictionary() -> Arc<FstDictionary> {
    // Convert the annotated mutable dictionary to FST format
    Arc::new((**POLISH_ANNOTATED_DICT).clone().into())
}

/// Returns the main curated Polish dictionary.
///
/// This uses the annotated dictionary which includes morphological annotations
/// for grammar analysis.
pub fn curated_polish_dictionary() -> Arc<FstDictionary> {
    polish_dictionary()
}

/// Returns the mutable Polish dictionary for annotation processing.
///
/// This is primarily used internally for annotation-based grammar checking.
pub fn mutable_polish_dictionary() -> Arc<MutableDictionary> {
    (*POLISH_ANNOTATED_DICT).clone()
}
