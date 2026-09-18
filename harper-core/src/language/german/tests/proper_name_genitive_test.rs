//! German puts a bare `-s` on a proper name to form the genitive — *Rembrandts
//! Werke*, *Orwells Roman*, *Maximilians Nachfolge* — and the name itself is
//! unchanged otherwise. Harper had the names and not the genitives, so 48 of
//! them were reported as misspellings across the archived corpus.
//!
//! The rule is hunspell de_DE's `SFX S`, on Harper's `H`. It is deliberately not
//! `b`, which is the same surface rule but declares both the result and the base
//! a **plural** noun: true for `Autos`, wrong for `Großbritanniens`.
//!
//! `H` is given only to capitalized entries, and that restriction is the whole
//! reason it is affordable. On all 45545 entries hunspell marks it cost 184
//! detections in `just language-recall german` — `anderen -> annderen`,
//! `jeweils -> jeweills`, `Anfang -> Annfang`, typos of the most frequent words
//! there are — because every new form is also a compound element. Capitalized
//! entries alone keep a third of the precision gain at a cost of one detection.
//!
//! What it cannot reach is a name that is not an entry. `Österreich`, `Goethe`
//! and `Peter` are spelled correctly only because the compound checker takes
//! them apart, so there is nothing for a flag to attach to and their genitives
//! are still missing.

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::linting::new_curated_german;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::Linter;
    use crate::spell::Dictionary;

    /// The genitives the corpus actually used.
    #[test]
    fn proper_name_genitives_are_words() {
        let dict = curated_german_dictionary();

        for word in [
            "Maximilians",
            "Rembrandts",
            "Orwells",
            "Montaignes",
            "Grillparzers",
            "Husserls",
            "Edwards",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is the genitive of a name the dictionary has"
            );
        }
    }

    /// A genitive singular is not a plural, which is why this is not on `b`.
    #[test]
    fn the_genitive_is_not_called_a_plural() {
        use crate::language::morphology::{MorphologyExt, Number};

        let dict = curated_german_dictionary();

        let meta = dict.get_word_metadata_str("Rembrandts").unwrap();
        assert!(meta.is_noun(), "'Rembrandts' is a noun");
        assert_ne!(
            meta.get_noun_number(),
            Some(Number::Plural),
            "'Rembrandts' is a genitive singular, not a plural"
        );
    }

    /// The restriction that makes it affordable: lower-case common nouns did not
    /// get the flag, so the typos that a bare `-s` on everything let through are
    /// still caught.
    #[test]
    fn frequent_typos_are_still_caught() {
        let dict = curated_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::default(), dict);

        for typo in ["annderen", "jeweills", "Annfang", "daanach"] {
            let document = Document::new(
                &format!("Das war {typo} gemeint."),
                &PlainGerman,
                &curated_german_dictionary(),
            );
            let lints = linter.lint(&document);
            assert!(
                lints
                    .iter()
                    .any(|l| l.span.get_content_string(&document.get_source().to_vec()) == typo),
                "'{typo}' is a misspelling and must still be flagged"
            );
        }
    }

    /// End to end.
    #[test]
    fn proper_name_genitives_produce_no_lints() {
        let dict = curated_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::default(), dict);

        let document = Document::new(
            "Rembrandts Werke und Maximilians Nachfolge.",
            &PlainGerman,
            &curated_german_dictionary(),
        );

        let lints = linter.lint(&document);
        assert!(
            lints.is_empty(),
            "proper-name genitives should be accepted, got {lints:?}"
        );
    }
}
