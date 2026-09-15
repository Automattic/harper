//! German dictionary based on the LanguageTool/igerman98 Hunspell word lists.
//!
//! The word list is derived from the igerman98 dictionary (GPLv2/GPLv3),
//! using annotated dictionary format for comprehensive coverage.
use crate::spell::rune::{AttributeList, parse_word_list};
use crate::spell::word_map::WordMap;
use crate::spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary};
use std::sync::{Arc, LazyLock};

use super::compound_aware_dict::CompoundAwareDictionary;
use super::compound_checker::CompoundChecker;

/// Load the German word list for lazy compound checking
fn load_german_word_list() -> Vec<crate::spell::rune::word_list::AnnotatedWord> {
    parse_word_list(include_str!("../dictionary.dict"))
        .expect("Failed to parse German dictionary word list")
}

/// Build the base German dictionary, without pre-generated compounds.
///
/// Expanding the affixes yields a little over a million entries, so this goes
/// straight from the expanded [`WordMap`] into [`FstDictionary`]. Routing it
/// through a [`MutableDictionary`] first would hold a second full copy of the
/// dictionary at the same time, and `FstDictionary` keeps one internally anyway.
fn build_german_base_dict(
    word_list: &[crate::spell::rune::word_list::AnnotatedWord],
) -> FstDictionary {
    let attr_list = AttributeList::parse(include_str!("../annotations.json"))
        .expect("Failed to parse German dictionary attribute list");

    let mut word_map = WordMap::default();
    attr_list.expand_annotated_words(word_list.iter().cloned(), &mut word_map);

    FstDictionary::new(
        word_map
            .into_iter()
            .map(|entry| (entry.canonical_spelling, entry.metadata))
            .collect(),
    )
}

// Word list for lazy compound checking (shared between all German dictionary components)
// This is initialized first to avoid duplicate parsing
static GERMAN_WORD_LIST: LazyLock<Vec<crate::spell::rune::word_list::AnnotatedWord>> =
    LazyLock::new(load_german_word_list);

/// The parsed base word list, with its property flags still attached.
///
/// Used by [`super::lexical_classes`] to derive the closed lexical classes the
/// capitalization linter needs, without paying for affix expansion or a
/// dictionary lookup.
pub(super) fn german_word_list() -> &'static [crate::spell::rune::word_list::AnnotatedWord] {
    &GERMAN_WORD_LIST
}

// Base dictionary without pre-generated compounds (FST for fast lookups).
static GERMAN_BASE_DICT: LazyLock<Arc<FstDictionary>> =
    LazyLock::new(|| Arc::new(build_german_base_dict(&GERMAN_WORD_LIST)));

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

// There are only three German dictionaries, built once each and then shared:
//
//   `GERMAN_BASE_DICT`           the expanded word list, as an FST
//   `GERMAN_COMPOUND_AWARE_DICT` the same, plus lazy compound decomposition
//   `GERMAN_COMBINED_DICT`       the compound-aware one behind `MergedDictionary`
//
// The accessors below are the historical names callers already use. Each is an
// `Arc` clone of one of those three; none of them builds anything.

/// The base German dictionary as an FST: fast lookups, fuzzy matching and prefix
/// search over the expanded word list.
///
/// Compound words are not in here. Use [`combined_german_dictionary`] when
/// compounds have to be recognised.
pub fn base_german_dictionary_fst() -> Arc<FstDictionary> {
    (*GERMAN_BASE_DICT).clone()
}

/// Alias for [`base_german_dictionary_fst`].
pub fn german_dictionary() -> Arc<FstDictionary> {
    base_german_dictionary_fst()
}

/// Alias for [`base_german_dictionary_fst`].
pub fn german_dictionary_fst() -> Arc<FstDictionary> {
    base_german_dictionary_fst()
}

/// Alias for [`base_german_dictionary_fst`].
pub fn curated_german_dictionary() -> Arc<FstDictionary> {
    base_german_dictionary_fst()
}

/// Alias for [`base_german_dictionary_fst`]. The morphological annotations live
/// on the entries themselves, so there is no separate annotated dictionary.
pub fn annotated_german_dictionary() -> Arc<FstDictionary> {
    base_german_dictionary_fst()
}

/// Alias for [`base_german_dictionary_fst`].
pub fn compound_aware_german_fst_dictionary() -> Arc<FstDictionary> {
    base_german_dictionary_fst()
}

/// The base German dictionary in mutable form, for bulk reads of the entries and
/// their metadata.
///
/// [`FstDictionary`] already keeps a [`MutableDictionary`] next to its FST, so
/// this shares that one. Converting instead would copy all million-odd entries.
pub fn base_german_dictionary() -> Arc<MutableDictionary> {
    base_german_dictionary_fst().as_mutable()
}

/// Alias for [`base_german_dictionary`].
pub fn mutable_german_dictionary() -> Arc<MutableDictionary> {
    base_german_dictionary()
}

/// The compound-aware dictionary behind a [`MergedDictionary`], which is the form
/// the linters and `Document::new` take.
pub fn combined_german_dictionary() -> Arc<MergedDictionary> {
    (*GERMAN_COMBINED_DICT).clone()
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
