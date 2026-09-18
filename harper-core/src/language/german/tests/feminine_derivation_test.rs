//! German derives a feminine personal noun from the masculine one with `-in`
//! (*Movierung*): `Lehrer` -> `Lehrerin`, `Biologe` -> `Biologin`, plural
//! `-innen`. It is fully productive, and it was missing from the model
//! entirely — `Sprecherin`, `Ministerin`, `Politikerin`, `Regisseurin` and
//! `Professorin` were all reported as misspellings while their masculine forms
//! were in the dictionary.
//!
//! The rule is hunspell de_DE's `SFX F`, split across two Harper flags the way
//! `-ung`/`-ungen` are split across `7` and `8`: `K` builds the singular and `L`
//! the plural, so the plural forms carry a plural reading rather than borrowing
//! the singular's. `scripts/mirror_hunspell_flag.py --from F --to KL` puts them
//! on the entries igerman98 marks, verifying every generated form first.
//!
//! `K` and `L` were previously an uppercase compound `-n` and `-en` interfix.
//! No entry carried either, and the compound checker never read them — it reads
//! the lowercase `k` and `l`.

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::linting::new_curated_german;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::Linter;
    use crate::spell::Dictionary;

    /// The derivation reaches the agent nouns the corpus actually used.
    #[test]
    fn feminine_agent_nouns_are_words() {
        let dict = curated_german_dictionary();

        for word in [
            "Sprecherin",
            "Ministerin",
            "Politikerin",
            "Regisseurin",
            "Professorin",
            "Kandidatin",
            "Journalistin",
            "Gitarristin",
            "Lehrerin",
            "Biologin",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is a feminine derivation of a noun the dictionary has"
            );
        }
    }

    /// And so does the plural.
    #[test]
    fn feminine_plurals_are_words() {
        let dict = curated_german_dictionary();

        for word in [
            "Lehrerinnen",
            "Sprecherinnen",
            "Ministerinnen",
            "Biologinnen",
            "Journalistinnen",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is the plural of a feminine derivation"
            );
        }
    }

    /// Both flags say the derivation is feminine, which is what the masculine
    /// entry cannot say: `näher` carries no gender at all, and `näherin` now
    /// does.
    #[test]
    fn the_derivation_is_feminine() {
        use crate::language::morphology::{Gender, MorphologyExt};

        let dict = curated_german_dictionary();

        for word in ["Lehrerin", "Lehrerinnen", "Ministerin", "Ministerinnen"] {
            let meta = dict.get_word_metadata_str(word).unwrap();
            assert!(meta.is_noun(), "'{word}' is a noun");
            assert_eq!(
                meta.get_noun_gender(),
                Some(Gender::Feminine),
                "'{word}' is a feminine derivation and should say so"
            );
        }
    }

    /// And `L` says its forms are plural, which is why it is a flag of its own.
    ///
    /// Only the plural is asserted. The singular cannot be asserted *not* to be
    /// plural: a slice of `dictionary.dict` stores capitalized nouns in lower
    /// case, several of those lower-case entries are themselves `-in` forms
    /// carrying the `Y` plural affix, and a word's readings are the union of
    /// everything that produces it. `näherin/~~NhY` is one such entry.
    #[test]
    fn the_plural_is_marked_plural() {
        use crate::language::morphology::{MorphologyExt, Number};

        let dict = curated_german_dictionary();

        for word in ["Lehrerinnen", "Ministerinnen", "Biologinnen"] {
            assert_eq!(
                dict.get_word_metadata_str(word).unwrap().get_noun_number(),
                Some(Number::Plural),
                "'{word}' is a plural form and should say so"
            );
        }
    }

    /// German also writes the derivation with an interior capital. igerman98
    /// lists those spellings, so Harper accepts them too.
    #[test]
    fn interior_capital_spelling_is_accepted() {
        let dict = curated_german_dictionary();

        for word in ["LehrerIn", "LehrerInnen", "MinisterIn"] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is the interior-capital spelling and igerman98 lists it"
            );
        }
    }

    /// End to end: a sentence full of them draws no lint.
    #[test]
    fn feminine_derivations_produce_no_lints() {
        let dict = curated_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::default(), dict);

        let document = Document::new(
            "Die Sprecherin und die Ministerin trafen die Regisseurin. \
             Die Journalistinnen befragten die Kandidatin.",
            &PlainGerman,
            &curated_german_dictionary(),
        );

        let lints = linter.lint(&document);
        assert!(
            lints.is_empty(),
            "feminine derivations should be accepted, got {lints:?}"
        );
    }
}
