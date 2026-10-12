//! Closed lexical classes read straight out of `dictionary.dict`.
//!
//! `GermanNounCapitalization` needs to reject spelled-out numerals, unit
//! abbreviations and lower-case foreign terms before it consults the dictionary
//! proper. These used to be `const &[&str]` tables in the linter, which meant
//! adding a word required editing Rust. They are now dictionary data, marked
//! with the property flags `1` (numeral), `2` (abbreviation) and `3` (foreign
//! term) -- see `annotations.json`.
//!
//! The sets are built from the *base* word list rather than the expanded runtime
//! dictionary, for two reasons:
//!
//! * The flags live on the base entries, so no affix expansion is needed.
//! * It keeps the linter's cheap pre-dictionary reject cheap. Consulting
//!   `CompoundAwareDictionary::get_word_metadata` for every candidate token would
//!   take a global mutex and run a compound decomposition on every miss --
//!   exactly what these classes exist to avoid.
//!
//! Each set is a process-wide `LazyLock`, so the scan happens once no matter how
//! often lint groups are rebuilt.

use hashbrown::HashSet;
use std::sync::LazyLock;

use crate::spell::rune::word_list::AnnotatedWord;

/// Property flag marking a spelled-out cardinal numeral (`zwei`, `hundert`).
pub const NUMERAL_FLAG: char = '1';
/// Property flag marking a unit abbreviation (`km`, `kWh`).
pub const ABBREVIATION_FLAG: char = '2';
/// Property flag marking a lower-case Latin/Greek term (`facto`, `sapiens`).
pub const FOREIGN_TERM_FLAG: char = '3';
/// Property flag marking a lower-case word whose noun homograph is capitalized
/// (`ist` and das `Ist`).
pub const LOWERCASE_NON_NOUN_FLAG: char = '^';
/// Property flag marking a strong verb's present form or imperative on the
/// changed stem (`gibt`, `lies`, `fährt`).
pub const STRONG_PRESENT_FLAG: char = '%';
/// Property flag marking a noun plural whose dative adds `-n` (`Kinder`).
pub const PLURAL_FORM_FLAG: char = '&';
/// Property flag marking a plurale tantum (`Leute`, `Eltern`).
pub const PLURALE_TANTUM_FLAG: char = '+';

fn collect_flagged(words: &[AnnotatedWord], flag: char) -> HashSet<String> {
    words
        .iter()
        .filter(|word| word.annotations.contains(&flag))
        .map(|word| word.letters.iter().collect::<String>().to_lowercase())
        .collect()
}

/// The sets of words that carry one of the class flags, collected in one pass over the word list.
pub struct LexicalClasses {
    numerals: HashSet<String>,
    unit_abbreviations: HashSet<String>,
    foreign_terms: HashSet<String>,
    lowercase_non_nouns: HashSet<String>,
    strong_present_forms: HashSet<String>,
    plural_forms: HashSet<String>,
    pluralia_tantum: HashSet<String>,
}

impl LexicalClasses {
    pub(super) fn from_word_list(words: &[AnnotatedWord]) -> Self {
        Self {
            numerals: collect_flagged(words, NUMERAL_FLAG),
            unit_abbreviations: collect_flagged(words, ABBREVIATION_FLAG),
            foreign_terms: collect_flagged(words, FOREIGN_TERM_FLAG),
            lowercase_non_nouns: collect_flagged(words, LOWERCASE_NON_NOUN_FLAG),
            strong_present_forms: collect_flagged(words, STRONG_PRESENT_FLAG),
            plural_forms: collect_flagged(words, PLURAL_FORM_FLAG),
            pluralia_tantum: collect_flagged(words, PLURALE_TANTUM_FLAG),
        }
    }
}

/// Spelled-out German cardinal numbers. In running text these are lower case,
/// so they are never flagged; used attributively they *license* a following noun
/// (*"die drei Streifen"*).
pub static NUMERALS: LazyLock<&'static HashSet<String>> =
    LazyLock::new(|| &super::german_dict::german_lexical_classes().numerals);

/// Unit abbreviations that are written lower case and must not be "corrected".
pub static UNIT_ABBREVIATIONS: LazyLock<&'static HashSet<String>> =
    LazyLock::new(|| &super::german_dict::german_lexical_classes().unit_abbreviations);

/// Latin / Greek etymology words that appear lower case in German prose
/// ("von lateinisch *scientia*", "de facto", "Homo *sapiens*"). They are in the
/// dictionary so the spell checker accepts them, but they are not
/// miscapitalized German nouns.
pub static FOREIGN_TERMS: LazyLock<&'static HashSet<String>> =
    LazyLock::new(|| &super::german_dict::german_lexical_classes().foreign_terms);

/// Words whose lower-case spelling is not a noun, although a capitalized noun of
/// the same letters is: `ist` and das `Ist`, `gut` and das `Gut`.
///
/// Lookups ignore case, so `ist` arrives carrying the noun reading of `Ist` and
/// the merged metadata cannot say which spelling it came from. The `^` flag on
/// the lower-case entry can.
pub static LOWERCASE_NON_NOUNS: LazyLock<&'static HashSet<String>> =
    LazyLock::new(|| &super::german_dict::german_lexical_classes().lowercase_non_nouns);

/// The present forms and imperatives of strong verbs on the changed stem:
/// `gib`, `gibt`, `gibst`, `tritt`, `fährt`, `hältst`. The word list marks each
/// one; see `build_german_dictionary.py`.
pub static STRONG_PRESENT_FORMS: LazyLock<&'static HashSet<String>> =
    LazyLock::new(|| &super::german_dict::german_lexical_classes().strong_present_forms);

/// Noun plurals that do not end in `-n` or `-s` and so take `-n` in the
/// dative: `Kinder`, `Freunde`, `Bücher`. Kept in their capitalized spelling;
/// the word list marks them `&`.
pub static PLURAL_FORMS: LazyLock<&'static HashSet<String>> =
    LazyLock::new(|| &super::german_dict::german_lexical_classes().plural_forms);

/// Nouns that only exist in the plural: `Leute`, `Eltern`, `Ferien`. Read
/// from the flag rather than the merged number, which an `E` beside the `+`
/// widens to both numbers (`leute/~~NhE+`).
pub static PLURALIA_TANTUM: LazyLock<&'static HashSet<String>> =
    LazyLock::new(|| &super::german_dict::german_lexical_classes().pluralia_tantum);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numerals_cover_the_basic_cardinals() {
        for word in [
            "null", "eins", "zwei", "drei", "sieben", "zwölf", "zwanzig", "hundert", "tausend",
            "million", "dutzend",
        ] {
            assert!(
                NUMERALS.contains(word),
                "{word} should carry the numeral flag in dictionary.dict"
            );
        }
    }

    #[test]
    fn unit_abbreviations_cover_common_units() {
        for word in [
            "km", "kg", "mg", "cm", "kwh", "hz", "ghz", "psi", "mio", "mrd",
        ] {
            assert!(
                UNIT_ABBREVIATIONS.contains(word),
                "{word} should carry the abbreviation flag in dictionary.dict"
            );
        }
    }

    #[test]
    fn foreign_terms_cover_the_latin_vocabulary() {
        for word in [
            "facto", "sapiens", "scientia", "circa", "alias", "versus", "vitae",
        ] {
            assert!(
                FOREIGN_TERMS.contains(word),
                "{word} should carry the foreign-term flag in dictionary.dict"
            );
        }
    }

    /// The classes are disjoint from ordinary vocabulary; a plain noun must not
    /// leak into any of them.
    #[test]
    fn ordinary_nouns_are_not_in_any_class() {
        for word in ["haus", "garten", "freiheit"] {
            assert!(!NUMERALS.contains(word));
            assert!(!UNIT_ABBREVIATIONS.contains(word));
            assert!(!FOREIGN_TERMS.contains(word));
        }
    }

    #[test]
    fn classes_are_non_empty_and_plausibly_sized() {
        // Guards against a flag being dropped from annotations.json or the
        // dictionary, which would silently disable the linter's reject path.
        assert!(NUMERALS.len() >= 30, "numerals: {}", NUMERALS.len());
        assert!(
            UNIT_ABBREVIATIONS.len() >= 20,
            "units: {}",
            UNIT_ABBREVIATIONS.len()
        );
        assert!(
            FOREIGN_TERMS.len() >= 15,
            "foreign: {}",
            FOREIGN_TERMS.len()
        );
    }
}
