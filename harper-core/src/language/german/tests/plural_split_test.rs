//! `Y` was documented as "Noun plural -n/-en" and meant it literally: every
//! entry carrying it got **both** `word + "n"` and `word + "en"`. A German noun
//! takes one or the other — `Diagnose` → `Diagnosen`, `Aufklärung` →
//! `Aufklärungen` — so for nearly every one of the hundred thousand entries that
//! carried it, one of the two was not a word. Many take neither, because the
//! plural umlauts (`Arzt` → `Ärzte`), doubles an `s` (`Ergebnis` →
//! `Ergebnisse`) or does not exist (`Chemie`).
//!
//! Of the 100842 entries igerman98 can judge: 29065 take only `-n`, 23196 only
//! `-en`, 31 both, 48550 neither. So `-n` moved to `E` and `Y` narrowed to
//! `-en`, and `scripts/split_german_plural_n.py` gave each entry whichever
//! igerman98 says it takes — about 148000 generated non-words fewer.
//!
//! An entry igerman98 does not list at all keeps both, so the compounds it
//! composes rather than lists are untouched.

#[cfg(test)]
mod tests {
    use crate::language::german::spell::curated_german_dictionary;
    use crate::language::morphology::{MorphologyExt, Number};
    use crate::spell::Dictionary;

    /// The plurals German actually forms are still there.
    #[test]
    fn real_plurals_survive() {
        let dict = curated_german_dictionary();

        for word in [
            "Diagnosen",
            "Fragen",
            "Aufklärungen",
            "Behandlungen",
            "Studenten",
            "Befunden",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is a real German plural and must survive the split"
            );
        }
    }

    /// The other half of each pair was never a word.
    #[test]
    fn the_half_that_was_never_a_word_is_gone() {
        let dict = curated_german_dictionary();

        for word in [
            "Diagnoseen",
            "Frageen",
            "Aufklärungn",
            "Behandlungn",
            "Arztn",
            "Arzten",
            "Chemien",
            "Chemieen",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_none(),
                "'{word}' is not a German plural and should not be in the dictionary"
            );
        }
    }

    /// Both flags still say "plural noun", which the capitalization linter leans
    /// on: for a word ending in `-e` it only treats a noun reading as real when
    /// the entry carries gender or number.
    #[test]
    fn both_halves_still_mark_a_plural() {
        let dict = curated_german_dictionary();

        for word in ["Diagnosen", "Aufklärungen"] {
            let meta = dict.get_word_metadata_str(word).unwrap();
            assert!(meta.is_noun(), "'{word}' is a noun");
            assert_eq!(
                meta.get_noun_number(),
                Some(Number::Plural),
                "'{word}' is a plural and should say so"
            );
        }
    }

    /// And the base forms keep the reading that makes `das Deutsche` work.
    #[test]
    fn bases_keep_their_noun_agreement() {
        let dict = curated_german_dictionary();

        let meta = dict.get_word_metadata_str("deutsche").unwrap();
        assert!(meta.is_noun() && meta.is_adjective(), "'deutsche' is both");
        assert!(
            meta.has_noun_agreement(),
            "the capitalization linter needs gender or number on a bare -e noun"
        );
    }
}
