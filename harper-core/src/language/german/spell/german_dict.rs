//! German dictionary based on the LanguageTool/igerman98 Hunspell word lists.
//!
//! The word list is derived from the igerman98 dictionary (GPLv2/GPLv3),
//! using annotated dictionary format for comprehensive coverage.
use crate::spell::rune::{AttributeList, parse_word_list};
use crate::spell::word_map::WordMap;
use crate::spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary};
use std::sync::{Arc, LazyLock, Mutex};

use super::compound_aware_dict::CompoundAwareDictionary;
use super::compound_checker::CompoundChecker;

fn load_german_fst_dict() -> Arc<FstDictionary> {
    // Convert the annotated dictionary to FST format for backward compatibility
    Arc::new((*load_german_annotated_dict()).clone().into())
}

fn load_german_annotated_dict() -> Arc<MutableDictionary> {
    // Delegate to base dict to avoid O(n^2) memory explosion from compound pre-generation
    // Compound words are now checked lazily via compound_aware_german_dictionary()
    load_german_base_dict_from_word_list(&GERMAN_WORD_LIST)
}

/// Load the German word list for lazy compound checking
fn load_german_word_list() -> Vec<crate::spell::rune::word_list::AnnotatedWord> {
    parse_word_list(include_str!("../dictionary.dict"))
        .expect("Failed to parse German dictionary word list")
}

/// Load the base German dictionary without pre-generated compounds
fn load_german_base_dict_from_word_list(
    word_list: &[crate::spell::rune::word_list::AnnotatedWord],
) -> Arc<MutableDictionary> {
    let attr_list = AttributeList::parse(include_str!("../annotations.json"))
        .expect("Failed to parse German dictionary attribute list");

    // Create word map and expand annotated words (but don't generate compounds)
    let mut word_map = WordMap::default();
    attr_list.expand_annotated_words(word_list.iter().cloned(), &mut word_map);

    // Create the MutableDictionary from the populated word map
    let mut dict = MutableDictionary::new();
    for entry in word_map.into_iter() {
        dict.append_word(entry.canonical_spelling, entry.metadata);
    }

    Arc::new(dict)
}

// Word list for lazy compound checking (shared between all German dictionary components)
// This is initialized first to avoid duplicate parsing
static GERMAN_WORD_LIST: LazyLock<Vec<crate::spell::rune::word_list::AnnotatedWord>> =
    LazyLock::new(load_german_word_list);

// Base dictionary without pre-generated compounds (FST for fast lookups)
static GERMAN_BASE_DICT: LazyLock<Arc<FstDictionary>> = LazyLock::new(|| {
    Arc::new(
        (*load_german_base_dict_from_word_list(&GERMAN_WORD_LIST))
            .clone()
            .into(),
    )
});

// Annotated dictionary using Rune format
static GERMAN_ANNOTATED_DICT: LazyLock<Arc<MutableDictionary>> =
    LazyLock::new(load_german_annotated_dict);

// Compound checker for lazy compound checking
static GERMAN_COMPOUND_CHECKER: LazyLock<Arc<Mutex<CompoundChecker>>> = LazyLock::new(|| {
    let mut checker = CompoundChecker::new(&GERMAN_WORD_LIST);
    checker.set_base_dictionary(Arc::clone(&GERMAN_BASE_DICT));
    Arc::new(Mutex::new(checker))
});

// Compound-aware dictionary using lazy compound checking
static GERMAN_COMPOUND_AWARE_DICT: LazyLock<Arc<CompoundAwareDictionary>> = LazyLock::new(|| {
    let base_dict = Arc::clone(&*GERMAN_BASE_DICT);
    let word_list = &*GERMAN_WORD_LIST;
    let compound_checker = CompoundChecker::new(word_list);

    Arc::new(CompoundAwareDictionary::new(base_dict, compound_checker))
});

// Combined dictionary: compound-aware dictionary for memory efficiency
// This provides both word coverage and metadata with lazy compound checking
static GERMAN_COMBINED_DICT: LazyLock<Arc<MergedDictionary>> = LazyLock::new(|| {
    use std::sync::Arc;

    let mut merged = MergedDictionary::new();

    // Add compound-aware dictionary - provides base words + lazy compound checking
    merged.add_dictionary(Arc::clone(&*GERMAN_COMPOUND_AWARE_DICT) as Arc<dyn Dictionary>);

    Arc::new(merged)
});

/// Returns a shared reference to the German FstDictionary.
///
/// The dictionary is loaded and built once on first access, then cached for the
/// lifetime of the process. This provides fuzzy matching, prefix search, and
/// all other `Dictionary` trait capabilities.
///
/// Note: This uses the compound-aware dictionary for memory efficiency.
/// Compound words are checked lazily rather than pre-generated.
pub fn german_dictionary() -> Arc<FstDictionary> {
    compound_aware_german_fst_dictionary()
}

/// Returns a shared reference to the annotated German dictionary.
///
/// This dictionary includes morphological annotations for German grammar analysis.
/// Note: For memory efficiency, this now uses the base dictionary without
/// pre-generating all compound combinations. Use compound_aware_german_dictionary()
/// for full compound word support with lazy checking.
pub fn annotated_german_dictionary() -> Arc<FstDictionary> {
    // Use base dictionary for memory efficiency - avoids O(n²) compound generation
    base_german_dictionary_fst()
}

/// Returns the main curated German dictionary.
///
/// Uses the compound-aware dictionary which provides comprehensive word coverage
/// with lazy compound checking to avoid memory explosion from pre-generating all compounds.
/// This provides both word coverage and metadata in a memory-efficient way.
pub fn curated_german_dictionary() -> Arc<FstDictionary> {
    // Use the pre-built FST base dictionary directly
    base_german_dictionary_fst()
}

/// Returns the compound-aware German FST dictionary using lazy compound checking.
///
/// This dictionary provides the base words in FST format for memory efficiency.
/// Compound words are checked lazily through the compound-aware dictionary
/// (accessible via compound_aware_german_dictionary() or combined_german_dictionary()).
/// This avoids the O(n²) memory explosion of pre-generating all compound combinations.
pub fn compound_aware_german_fst_dictionary() -> Arc<FstDictionary> {
    // Return the base dictionary as FST format for memory efficiency
    // Note: For full compound word support, use combined_german_dictionary()
    // which uses CompoundAwareDictionary with lazy compound checking
    base_german_dictionary_fst()
}

/// Returns the mutable German dictionary for annotation processing.
///
/// This is primarily used internally for annotation-based grammar checking.
/// Uses the base dictionary without pre-generated compounds for memory efficiency.
pub fn mutable_german_dictionary() -> Arc<MutableDictionary> {
    // Use base dictionary directly - GERMAN_ANNOTATED_DICT now delegates to load_german_base_dict()
    base_german_dictionary()
}

/// Returns the combined German dictionary with comprehensive word coverage and annotations.
///
/// This dictionary uses the compound-aware dictionary which provides base words
/// with lazy compound checking. This maintains full compound word support while
/// avoiding the O(n²) memory explosion of pre-generating all compound combinations.
pub fn combined_german_dictionary() -> Arc<MergedDictionary> {
    (*GERMAN_COMBINED_DICT).clone()
}

/// Returns the base German dictionary without pre-generated compounds.
///
/// This dictionary contains only the base words from the German dictionary
/// without any compound word generation. It's used as the foundation for
/// lazy compound checking.
pub fn base_german_dictionary() -> Arc<MutableDictionary> {
    // Convert FST back to MutableDictionary for API compatibility
    // Note: base_german_dictionary_fst() returns the FST version directly
    Arc::new((*base_german_dictionary_fst()).clone().into())
}

/// Returns the base German dictionary as FST format (fast lookups).
pub fn base_german_dictionary_fst() -> Arc<FstDictionary> {
    (*GERMAN_BASE_DICT).clone()
}

/// Returns the FST German dictionary (alias for base_german_dictionary_fst).
pub fn german_dictionary_fst() -> Arc<FstDictionary> {
    (*GERMAN_BASE_DICT).clone()
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
