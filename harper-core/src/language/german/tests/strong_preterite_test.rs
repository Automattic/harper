//! German strong verbs ablaut, so their preterite cannot be derived from the
//! infinitive: `ziehen` gives `zog`, not `ziehte`. The conjugation flags built
//! on the infinitive therefore skip them entirely, and `stattfanden`,
//! `ausschieden`, `überließen` and `vorhielten` were reported as misspellings.
//!
//! igerman98 solves this by listing the preterite stem as a headword of its own
//! and inflecting it with `SFX Z`. Harper's `s` flag mirrors that rule, and
//! `harper-core/src/language/german/scripts/mirror_hunspell_flag.py --from Z --to s` puts it on the same entries.
//!
//! The stems were also mined as nouns (`zog/~~NhYr`), which made the personal
//! forms "appear to be a noun" mid-sentence. Stripping the noun property is not
//! enough on its own: the noun-plural affixes carry a plural reading of their
//! own, so those go too — see `harper-core/src/language/german/scripts/fix_german_pos_flags.py`.

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::linting::new_curated_german;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::Linter;
    use crate::spell::Dictionary;

    /// The personal forms the `s` flag builds are words.
    #[test]
    fn strong_preterite_forms_are_known_words() {
        let dict = curated_german_dictionary();

        for word in [
            "zog",
            "zogst",
            "zogt",
            "zogen",
            "schrieb",
            "schriebst",
            "schrieben",
            "stattfanden",
            "ausschieden",
            "überließen",
            "vorhielten",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is a strong preterite form and should be in the dictionary"
            );
        }
    }

    /// And they are verbs, not nouns. `zogt` had its own noun-mined entry
    /// (`zogt/~~NhYG`), so the flag on `zog` alone would not have reached it.
    #[test]
    fn strong_preterite_forms_are_not_nouns() {
        let dict = curated_german_dictionary();

        for word in ["zog", "zogt", "schrieb", "stattfand", "überließ"] {
            let meta = dict.get_word_metadata_str(word).unwrap();
            assert!(
                !meta.is_noun(),
                "'{word}' is a preterite verb form and must not carry a noun reading"
            );
        }
    }

    /// A lower-case entry standing in for a capitalized noun keeps its plural.
    /// `maß` is the preterite of `messen` *and* the only entry for the noun
    /// `Maß`; taking the plural affix off it deletes `Maße`.
    #[test]
    fn noun_homographs_keep_their_plural() {
        let dict = curated_german_dictionary();

        for word in ["Maße", "Sog", "Sogen"] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is a noun form and must survive the preterite retag"
            );
        }
    }

    /// End to end: a sentence of strong preterites draws no lint.
    #[test]
    fn strong_preterites_produce_no_lints() {
        let dict = curated_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::default(), dict);

        let document = Document::new(
            "Du zogst weiter, ihr zogt mit, sie zogen fort. Die Spiele stattfanden nicht.",
            &PlainGerman,
            &curated_german_dictionary(),
        );

        let lints = linter.lint(&document);
        assert!(
            lints.is_empty(),
            "strong preterite forms should be accepted, got {lints:?}"
        );
    }
}
