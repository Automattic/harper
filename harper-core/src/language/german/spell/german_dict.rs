//! German dictionary based on the LanguageTool/igerman98 Hunspell word lists.
//!
//! The word list is derived from the igerman98 dictionary (GPLv2/GPLv3),
//! expanded using Hunspell affix rules for comprehensive coverage.
//! It is embedded as gzip-compressed data and decompressed once at first use.
use crate::spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary};

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
static GERMAN_ANNOTATED_DICT: LazyLock<Arc<MutableDictionary>> =
    LazyLock::new(load_german_annotated_dict);

// Combined dictionary: FST dictionary (1.3M+ words) + annotated dictionary (237K words with metadata)
// The annotated dictionary is checked first for metadata, then the FST dictionary for word existence
static GERMAN_COMBINED_DICT: LazyLock<Arc<MergedDictionary>> = LazyLock::new(|| {
    use std::sync::Arc;

    let mut merged = MergedDictionary::new();

    // Add annotated dictionary FIRST - it has metadata for 237K words
    // This ensures that when a word exists in both, we get the annotation metadata
    merged.add_dictionary(Arc::clone(&*GERMAN_ANNOTATED_DICT) as Arc<dyn Dictionary>);

    // Add FST dictionary SECOND - it has 1.3M+ words but no explicit metadata
    // This provides comprehensive word coverage
    merged.add_dictionary(Arc::clone(&*GERMAN_FST_DICT) as Arc<dyn Dictionary>);

    Arc::new(merged)
});

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

/// Returns the main curated German dictionary.
///
/// Uses a merged dictionary combining:
/// 1. Annotated dictionary (237K words with explicit POS metadata)
/// 2. Large FST dictionary (1.3M+ words from Hunspell)
///
/// The annotated dictionary is checked first, so words that exist in both
/// will use the explicit metadata from the annotated dictionary.
/// This provides comprehensive word coverage while maintaining annotation
/// information for words that have it.
pub fn curated_german_dictionary() -> Arc<FstDictionary> {
    // For backward compatibility with LanguageModule trait, we return the FST dictionary
    // The annotated dictionary is used via the GERMAN_COMBINED_DICT for linters that need both
    german_dictionary()
}

/// Returns the mutable German dictionary for annotation processing.
///
/// This is primarily used internally for annotation-based grammar checking.
pub fn mutable_german_dictionary() -> Arc<MutableDictionary> {
    (*GERMAN_ANNOTATED_DICT).clone()
}

/// Returns the combined German dictionary with comprehensive word coverage and annotations.
///
/// This dictionary combines:
/// 1. Annotated dictionary (237K words with explicit POS metadata) - checked first
/// 2. Large FST dictionary (1.3M+ words from Hunspell) - checked second
///
/// Words that exist in both dictionaries will use the metadata from the annotated dictionary.
/// This provides comprehensive word coverage while maintaining annotation information.
pub fn combined_german_dictionary() -> Arc<MergedDictionary> {
    (*GERMAN_COMBINED_DICT).clone()
}
