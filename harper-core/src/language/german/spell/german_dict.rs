//! The German dictionary.
//!
//! The word list, `dictionary.dict`, is written by `scripts/build_german_dictionary.py` from the
//! German lexemes and proper names of Wikidata (CC0) and from word tables written for this
//! repository.
use crate::spell::rune::{AttributeList, parse_word_list};
use crate::spell::{Dictionary, MergedDictionary};
use std::sync::{Arc, LazyLock, Mutex};

use super::compact_dict::{CompactDictionary, expand_in_blocks};
use super::compound_aware_dict::CompoundAwareDictionary;
use super::compound_checker::CompoundChecker;

/// Load the German word list for lazy compound checking
fn load_german_word_list() -> Vec<crate::spell::rune::word_list::AnnotatedWord> {
    parse_word_list(include_str!("../dictionary.dict"))
        .expect("Failed to parse German dictionary word list")
}

/// Build the base German dictionary, without pre-generated compounds.
///
/// Expanding the affixes yields a little over 700 000 entries. Expanded into a
/// single [`WordMap`] they take 190 MB while building, and even as an
/// [`FstDictionary`](crate::spell::FstDictionary) 240 MB afterwards, so they go
/// straight into a [`CompactDictionary`] as the expansion finishes with them.
fn build_german_base_dict(
    word_list: &[crate::spell::rune::word_list::AnnotatedWord],
) -> CompactDictionary {
    let attr_list = AttributeList::parse(include_str!("../annotations.json"))
        .expect("Failed to parse German dictionary attribute list");

    expand_in_blocks(&attr_list, word_list)
}

/// How many threads the dictionary build may use. WebAssembly has none.
fn parallelism() -> usize {
    if cfg!(target_arch = "wasm32") {
        1
    } else {
        std::thread::available_parallelism().map_or(1, usize::from)
    }
}

/// Run two independent computations, side by side where threads exist.
fn join<A: Send, B: Send>(a: impl FnOnce() -> A + Send, b: impl FnOnce() -> B + Send) -> (A, B) {
    if parallelism() < 2 {
        return (a(), b());
    }
    std::thread::scope(|scope| {
        let handle = scope.spawn(b);
        let a = a();
        (a, handle.join().expect("German dictionary build panicked"))
    })
}

/// Everything the word list is read for, built in one pass.
///
/// The parsed word list is a third of the heap while the dictionary is being built
/// (a `CharString` and a flag vector per entry), and nothing needs it afterwards:
/// the expanded dictionary, the compound checker and the small sets below are all
/// derived from it once. So it is parsed here, used, and dropped.
struct GermanCore {
    base: Arc<CompactDictionary>,
    /// Taken by the compound-aware dictionary when it is first built.
    checker: Mutex<Option<CompoundChecker>>,
    stems: Arc<hashbrown::HashSet<crate::CharString>>,
    function_words: Arc<hashbrown::HashSet<crate::CharString>>,
    suffixed_elements: Arc<hashbrown::HashSet<crate::CharString>>,
    barred_elements: Arc<hashbrown::HashSet<crate::CharString>>,
    classes: super::lexical_classes::LexicalClasses,
}

static GERMAN_CORE: LazyLock<GermanCore> = LazyLock::new(|| {
    let word_list = load_german_word_list();
    let classes = super::lexical_classes::LexicalClasses::from_word_list(&word_list);
    // One after the other: built side by side, the two peaks add up, and the
    // dictionary build is already spread over the cores.
    let base = Arc::new(build_german_base_dict(&word_list));
    let checker = CompoundChecker::for_dictionary(&word_list);
    let [stems, function_words, suffixed_elements, barred_elements] = checker.shared_sets();
    GermanCore {
        base,
        checker: Mutex::new(Some(checker)),
        stems,
        function_words,
        suffixed_elements,
        barred_elements,
        classes,
    }
});

/// A freshly parsed copy of the word list, for the tests that compare it with the dictionary.
#[cfg(test)]
pub(super) fn german_word_list() -> Vec<crate::spell::rune::word_list::AnnotatedWord> {
    load_german_word_list()
}

/// The closed lexical classes read from the word list; see [`super::lexical_classes`].
pub(super) fn german_lexical_classes() -> &'static super::lexical_classes::LexicalClasses {
    &GERMAN_CORE.classes
}

/// The bare stems of the German word list: elements that are not words.
///
/// See the `stem_only` property in `annotations.json`. `GermanSpellCheck`
/// reads this when it decomposes a compound, so that `Messkabel` still works
/// although `mess` is not a word.
pub static GERMAN_STEMS: LazyLock<&'static hashbrown::HashSet<crate::CharString>> =
    LazyLock::new(|| &*GERMAN_CORE.stems);

/// The articles, pronouns and conjunctions of the German word list.
///
/// See the `function_word_set` documentation in
/// [`super::compound_checker`]. `GermanSpellCheck` decomposes compounds too and
/// reads the set from here, so both decompositions agree on what may end one.
pub static GERMAN_FUNCTION_WORDS: LazyLock<&'static hashbrown::HashSet<crate::CharString>> =
    LazyLock::new(|| &*GERMAN_CORE.function_words);

/// The derived feminine nouns of the German word list, which open a compound
/// only with the `-s-` interfix.
///
/// See the `suffixed_element_set` documentation in [`super::compound_checker`].
pub static GERMAN_SUFFIXED_ELEMENTS: LazyLock<&'static hashbrown::HashSet<crate::CharString>> =
    LazyLock::new(|| &*GERMAN_CORE.suffixed_elements);

/// The words of the German word list that never take part in a compound.
///
/// See `may_be_compound_element` in [`super::compound_checker`]. Read by
/// `GermanSpellCheck`, so both decompositions agree on what may be an element.
pub static GERMAN_BARRED_ELEMENTS: LazyLock<&'static hashbrown::HashSet<crate::CharString>> =
    LazyLock::new(|| &*GERMAN_CORE.barred_elements);

static GERMAN_BASE_DICT: LazyLock<Arc<CompactDictionary>> =
    LazyLock::new(|| Arc::clone(&GERMAN_CORE.base));

// Compound-aware dictionary using lazy compound checking
static GERMAN_COMPOUND_AWARE_DICT: LazyLock<Arc<CompoundAwareDictionary>> = LazyLock::new(|| {
    let base_dict = Arc::clone(&*GERMAN_BASE_DICT);
    let compound_checker = GERMAN_CORE
        .checker
        .lock()
        .unwrap()
        .take()
        .expect("the compound checker is taken once");

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
pub fn base_german_dictionary_fst() -> Arc<CompactDictionary> {
    (*GERMAN_BASE_DICT).clone()
}

/// Alias for [`base_german_dictionary_fst`].
pub fn german_dictionary() -> Arc<CompactDictionary> {
    base_german_dictionary_fst()
}

/// Alias for [`base_german_dictionary_fst`].
pub fn german_dictionary_fst() -> Arc<CompactDictionary> {
    base_german_dictionary_fst()
}

/// Alias for [`base_german_dictionary_fst`].
pub fn curated_german_dictionary() -> Arc<CompactDictionary> {
    base_german_dictionary_fst()
}

/// Alias for [`base_german_dictionary_fst`]. The morphological annotations live
/// on the entries themselves, so there is no separate annotated dictionary.
pub fn annotated_german_dictionary() -> Arc<CompactDictionary> {
    base_german_dictionary_fst()
}

/// Alias for [`base_german_dictionary_fst`].
pub fn compound_aware_german_fst_dictionary() -> Arc<CompactDictionary> {
    base_german_dictionary_fst()
}

/// The base German dictionary in mutable form, for bulk reads of the entries and
/// their metadata.
///
/// [`CompactDictionary`] already keeps a [`MutableDictionary`] next to its FST, so
/// this shares that one. Converting instead would copy all million-odd entries.
pub fn base_german_dictionary() -> Arc<CompactDictionary> {
    base_german_dictionary_fst()
}

/// Alias for [`base_german_dictionary`].
pub fn mutable_german_dictionary() -> Arc<CompactDictionary> {
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
