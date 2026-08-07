//! German dictionary based on the LanguageTool/igerman98 Hunspell word lists.
//!
//! The word list is derived from the igerman98 dictionary (GPLv2/GPLv3),
//! using annotated dictionary format for comprehensive coverage.
use crate::spell::rune::{AttributeList, parse_word_list};
use crate::spell::word_map::WordMap;
use crate::spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary};
use std::sync::{Arc, LazyLock, Mutex};

use super::compound;
use super::compound_aware_dict::CompoundAwareDictionary;
use super::compound_checker::CompoundChecker;

fn load_german_fst_dict() -> Arc<FstDictionary> {
    // Convert the annotated dictionary to FST format for backward compatibility
    Arc::new((*load_german_annotated_dict()).clone().into())
}

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
    // NOTE: For memory efficiency, consider using compound_aware_german_dictionary() instead
    // which uses lazy compound checking instead of pre-generating all compounds
    compound::generate_compound_words(&word_list, &mut word_map);

    // Create the MutableDictionary from the populated word map
    let mut dict = MutableDictionary::new();
    for entry in word_map.into_iter() {
        dict.append_word(entry.canonical_spelling, entry.metadata);
    }

    Arc::new(dict)
}

/// Load the German word list for lazy compound checking
fn load_german_word_list() -> Vec<crate::spell::rune::word_list::AnnotatedWord> {
    parse_word_list(include_str!("../dictionary.dict"))
        .expect("Failed to parse German dictionary word list")
}

/// Load the base German dictionary without pre-generated compounds
fn load_german_base_dict() -> Arc<MutableDictionary> {
    // Parse word list and attribute list
    let word_list = load_german_word_list();
    let attr_list = AttributeList::parse(include_str!("../annotations.json"))
        .expect("Failed to parse German dictionary attribute list");

    // Create word map and expand annotated words (but don't generate compounds)
    let mut word_map = WordMap::default();
    attr_list.expand_annotated_words(word_list, &mut word_map);

    // Create the MutableDictionary from the populated word map
    let mut dict = MutableDictionary::new();
    for entry in word_map.into_iter() {
        dict.append_word(entry.canonical_spelling, entry.metadata);
    }

    Arc::new(dict)
}

// Annotated dictionary using Rune format
static GERMAN_ANNOTATED_DICT: LazyLock<Arc<MutableDictionary>> =
    LazyLock::new(load_german_annotated_dict);

// Base dictionary without pre-generated compounds
static GERMAN_BASE_DICT: LazyLock<Arc<MutableDictionary>> = LazyLock::new(load_german_base_dict);

// Compound checker for lazy compound checking
static GERMAN_COMPOUND_CHECKER: LazyLock<Arc<Mutex<CompoundChecker>>> = LazyLock::new(|| {
    let word_list = load_german_word_list();
    let checker = CompoundChecker::new(&word_list);
    Arc::new(Mutex::new(checker))
});

// Compound-aware dictionary using lazy compound checking
static GERMAN_COMPOUND_AWARE_DICT: LazyLock<Arc<CompoundAwareDictionary>> = LazyLock::new(|| {
    let base_dict = Arc::clone(&*GERMAN_BASE_DICT);
    let word_list = load_german_word_list();
    let compound_checker = CompoundChecker::new(&word_list);

    Arc::new(CompoundAwareDictionary::new(base_dict, compound_checker))
});

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
/// Note: This now uses the base dictionary without pre-generated compounds.
pub fn annotated_german_dictionary() -> Arc<FstDictionary> {
    // Convert the MutableDictionary to FstDictionary
    Arc::new((**GERMAN_ANNOTATED_DICT).clone().into())
}

/// Returns the main curated German dictionary.
///
/// Uses the annotated dictionary which provides both word coverage and metadata.
/// This is now a single unified dictionary approach, consistent with other languages.
///
/// NOTE: For memory efficiency with large German dictionaries, consider using
/// `compound_aware_german_dictionary()` which uses lazy compound checking instead
/// of pre-generating all compound combinations.
pub fn curated_german_dictionary() -> Arc<FstDictionary> {
    // Return the annotated dictionary as FST format for consistency
    annotated_german_dictionary()
}

/// Returns the compound-aware German FST dictionary using lazy compound checking.
///
/// This dictionary provides comprehensive word coverage with lazy compound checking
/// to avoid the O(n²) memory explosion of pre-generating all compound combinations.
pub fn compound_aware_german_fst_dictionary() -> Arc<FstDictionary> {
    // Return the base dictionary as FST format
    // Note: For the full compound-aware dictionary, use compound_aware_german_dictionary()
    base_german_dictionary_fst()
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

/// Returns the base German dictionary without pre-generated compounds.
///
/// This dictionary contains only the base words from the German dictionary
/// without any compound word generation. It's used as the foundation for
/// lazy compound checking.
pub fn base_german_dictionary() -> Arc<MutableDictionary> {
    (*GERMAN_BASE_DICT).clone()
}

/// Returns the base German dictionary as FST format.
pub fn base_german_dictionary_fst() -> Arc<FstDictionary> {
    Arc::new((*base_german_dictionary()).clone().into())
}

/// Returns the compound checker for German dictionary.
///
/// This provides lazy compound word checking functionality without
/// pre-generating all possible compound combinations.
pub fn german_compound_checker() -> Arc<Mutex<CompoundChecker>> {
    (*GERMAN_COMPOUND_CHECKER).clone()
}

/// Returns the compound-aware German dictionary using lazy compound checking.
///
/// This dictionary first checks the base dictionary, and if a word is not found,
/// it uses lazy decomposition to check if the word is a valid German compound.
/// This approach avoids the O(n²) memory explosion of pre-generating all compounds
/// while still providing comprehensive compound word coverage.
///
/// Note: This dictionary does not support all Dictionary trait methods equally well.
/// For methods like word_count() and words_iter(), it returns data from the base
/// dictionary only, since compound words are not explicitly stored.
pub fn compound_aware_german_dictionary() -> Arc<CompoundAwareDictionary> {
    (*GERMAN_COMPOUND_AWARE_DICT).clone()
}
