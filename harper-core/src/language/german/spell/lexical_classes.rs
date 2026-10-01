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

fn collect_flagged(words: &[AnnotatedWord], flag: char) -> HashSet<String> {
    words
        .iter()
        .filter(|word| word.annotations.contains(&flag))
        .map(|word| word.letters.iter().collect::<String>().to_lowercase())
        .collect()
}

/// Spelled-out German cardinal numbers. In running text these are lower case,
/// so they are never flagged; used attributively they *license* a following noun
/// (*"die drei Streifen"*).
pub static NUMERALS: LazyLock<HashSet<String>> =
    LazyLock::new(|| collect_flagged(super::german_dict::german_word_list(), NUMERAL_FLAG));

/// Unit abbreviations that are written lower case and must not be "corrected".
pub static UNIT_ABBREVIATIONS: LazyLock<HashSet<String>> =
    LazyLock::new(|| collect_flagged(super::german_dict::german_word_list(), ABBREVIATION_FLAG));

/// Latin / Greek etymology words that appear lower case in German prose
/// ("von lateinisch *scientia*", "de facto", "Homo *sapiens*"). They are in the
/// dictionary so the spell checker accepts them, but they are not
/// miscapitalized German nouns.
pub static FOREIGN_TERMS: LazyLock<HashSet<String>> =
    LazyLock::new(|| collect_flagged(super::german_dict::german_word_list(), FOREIGN_TERM_FLAG));

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
