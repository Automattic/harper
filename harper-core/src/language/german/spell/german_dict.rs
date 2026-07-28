//! German dictionary based on the LanguageTool/igerman98 Hunspell word lists.
//!
//! The word list is derived from the igerman98 dictionary (GPLv2/GPLv3),
//! using annotated dictionary format for comprehensive coverage.
use crate::spell::rune::{AttributeList, parse_word_list};
use crate::spell::word_map::WordMap;
use crate::spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary};
use std::sync::{Arc, LazyLock};

use super::compound;

#[cfg(feature = "de")]
fn load_german_fst_dict() -> Arc<FstDictionary> {
    // Convert the annotated dictionary to FST format for backward compatibility
    Arc::new((*load_german_annotated_dict()).clone().into())
}

#[cfg(not(feature = "de"))]
fn load_german_fst_dict() -> Arc<FstDictionary> {
    Arc::new(FstDictionary::new(Vec::new()))
}

#[cfg(feature = "de")]
fn load_german_annotated_dict() -> Arc<MutableDictionary> {
    // Parse word list and attribute list
    let word_list = parse_word_list(include_str!("../dictionary.dict"))
        .expect("Failed to parse German dictionary word list");
    let attr_list = AttributeList::parse(include_str!("../annotations.json"))
        .expect("Failed to parse German dictionary attribute list");

    // Create word map and expand annotated words
    let mut word_map = WordMap::default();
    attr_list.expand_annotated_words(word_list.clone(), &mut word_map);

    // Generate German compound words from words with compound flags
    compound::generate_compound_words(&word_list, &mut word_map);

    // Create the MutableDictionary from the populated word map
    let mut dict = MutableDictionary::new();
    for entry in word_map.into_iter() {
        dict.append_word(entry.canonical_spelling, entry.metadata);
    }

    Arc::new(dict)
}

#[cfg(not(feature = "de"))]
fn load_german_annotated_dict() -> Arc<MutableDictionary> {
    Arc::new(MutableDictionary::new())
}

// Annotated dictionary using Rune format
static GERMAN_ANNOTATED_DICT: LazyLock<Arc<MutableDictionary>> =
    LazyLock::new(load_german_annotated_dict);

// Combined dictionary: annotated dictionary only (simplified approach)
// This provides both word coverage and metadata in a single dictionary
static GERMAN_COMBINED_DICT: LazyLock<Arc<MergedDictionary>> = LazyLock::new(|| {
    use std::sync::Arc;

    let mut merged = MergedDictionary::new();

    // Add annotated dictionary - it provides both word coverage and metadata
    merged.add_dictionary(Arc::clone(&*GERMAN_ANNOTATED_DICT) as Arc<dyn Dictionary>);

    Arc::new(merged)
});

/// Returns a shared reference to the German FstDictionary.
///
/// The dictionary is loaded and built once on first access, then cached for the
/// lifetime of the process. This provides fuzzy matching, prefix search, and
/// all other `Dictionary` trait capabilities.
///
/// Note: This now uses the annotated dictionary converted to FST format for consistency.
pub fn german_dictionary() -> Arc<FstDictionary> {
    load_german_fst_dict()
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
/// Uses the annotated dictionary which provides both word coverage and metadata.
/// This is now a single unified dictionary approach, consistent with other languages.
pub fn curated_german_dictionary() -> Arc<FstDictionary> {
    // Return the annotated dictionary as FST format for consistency
    annotated_german_dictionary()
}

/// Returns the mutable German dictionary for annotation processing.
///
/// This is primarily used internally for annotation-based grammar checking.
pub fn mutable_german_dictionary() -> Arc<MutableDictionary> {
    (*GERMAN_ANNOTATED_DICT).clone()
}

/// Returns the combined German dictionary with comprehensive word coverage and annotations.
///
/// This dictionary uses the annotated dictionary which provides both word coverage and metadata.
/// This is now a single unified dictionary approach, consistent with other languages.
pub fn combined_german_dictionary() -> Arc<MergedDictionary> {
    (*GERMAN_COMBINED_DICT).clone()
}
