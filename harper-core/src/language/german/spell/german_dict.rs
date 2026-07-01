//! German dictionary based on the LanguageTool/igerman98 Hunspell word lists.
//!
//! The word list is derived from the igerman98 dictionary (GPLv2/GPLv3),
//! expanded using Hunspell affix rules for comprehensive coverage.
//! It is embedded as gzip-compressed data and decompressed once at first use.
use crate::spell::{FstDictionary, MutableDictionary};

#[cfg(feature = "de")]
use crate::spell::embedded_dictionary::fst_dictionary_from_gzip_bytes;
use std::sync::{Arc, LazyLock};

#[cfg(feature = "de")]
fn load_german_fst_dict() -> Arc<FstDictionary> {
    Arc::new(fst_dictionary_from_gzip_bytes(include_bytes!(
        "../german_dictionary.dict.gz"
    )))
}

#[cfg(not(feature = "de"))]
fn load_german_fst_dict() -> Arc<FstDictionary> {
    Arc::new(FstDictionary::new(Vec::new()))
}

#[cfg(feature = "de")]
fn load_german_annotated_dict() -> Arc<MutableDictionary> {
    MutableDictionary::from_rune_files(
        include_str!("../dictionary.dict"),
        include_str!("../annotations.json"),
    )
    .map(Arc::new)
    .unwrap_or_else(|e| panic!("Failed to load German annotated dictionary: {}", e))
}

#[cfg(not(feature = "de"))]
fn load_german_annotated_dict() -> Arc<MutableDictionary> {
    Arc::new(MutableDictionary::new())
}

// Original FST dictionary for backward compatibility
static GERMAN_FST_DICT: LazyLock<Arc<FstDictionary>> = LazyLock::new(load_german_fst_dict);

// New annotated dictionary using Rune format
static GERMAN_ANNOTATED_DICT: LazyLock<Arc<MutableDictionary>> = LazyLock::new(load_german_annotated_dict);

/// Returns a shared reference to the original German FstDictionary.
///
/// The dictionary is loaded and built once on first access, then cached for the
/// lifetime of the process. This provides fuzzy matching, prefix search, and
/// all other `Dictionary` trait capabilities.
pub fn german_dictionary() -> Arc<FstDictionary> {
    (*GERMAN_FST_DICT).clone()
}

/// Returns a shared reference to the annotated German dictionary.
///
/// This dictionary includes morphological annotations for German grammar analysis.
pub fn annotated_german_dictionary() -> Arc<FstDictionary> {
    // Convert the MutableDictionary to FstDictionary
    Arc::new((**GERMAN_ANNOTATED_DICT).clone().into())
}

/// Returns the main curated German dictionary (currently uses the original FST dictionary).
///
/// For annotation-aware features, use `annotated_german_dictionary()` instead.
pub fn curated_german_dictionary() -> Arc<FstDictionary> {
    // Convert the annotated mutable dictionary to FST format
    Arc::new((**GERMAN_ANNOTATED_DICT).clone().into())
}

/// Returns the mutable German dictionary for annotation processing.
///
/// This is primarily used internally for annotation-based grammar checking.
pub fn mutable_german_dictionary() -> Arc<MutableDictionary> {
    (*GERMAN_ANNOTATED_DICT).clone()
}
