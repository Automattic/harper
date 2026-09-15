//! German closed-class words must not carry noun readings.
//!
//! `GermanNounCapitalization` flags a word with an unambiguous noun reading
//! wherever it appears. A large number of articles, pronouns, determiners,
//! prepositions and conjunctions were tagged as nouns in `dictionary.dict`
//! (`der/~~DzMpY` claimed both a masculine and a neuter noun with a plural), so
//! the linter wanted to capitalize ordinary function words.
//!
//! The closed-class flags are digits — `4` determiner, `5` pronoun, `6`
//! conjunction — because the corresponding letters `D`, `I` and `C` are also
//! affix rules and tagging a word with them would generate junk forms
//! (`mein` + `D` would produce `meinung`).

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::linting::new_curated_german;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::Linter;
    use crate::spell::Dictionary;

    /// Articles, pronouns, conjunctions and prepositions are not nouns.
    #[test]
    fn closed_class_words_carry_no_noun_reading() {
        let dict = curated_german_dictionary();

        for word in [
            // articles and determiners
            "der", "die", "das", "dem", "den", "des", "ein", "eine", "einen", "jede", "jeden",
            "jedes", "dieser", "diesem", "welche", "kein", "keine", // pronouns
            "er", "sie", "es", "wir", "ihr", "ich", "du", "mich", "mir", "dich", "wer",
            // conjunctions
            "und", "oder", "aber", "wenn", "dass", "weil", // prepositions
            "in", "von", "mit", "nach", "bei", "zu",
        ] {
            let metadata = dict
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("'{word}' should be in the German dictionary"));

            assert!(
                !metadata.is_noun(),
                "'{word}' is a closed-class function word and must not carry a noun \
                 reading; GermanNounCapitalization would try to capitalize it"
            );
        }
    }

    /// Removing the bogus flags must not remove the words themselves.
    #[test]
    fn closed_class_words_are_still_spelled_correctly() {
        let dict = curated_german_dictionary();

        for word in [
            "jeden", "jedem", "jede", "dieser", "diesem", "welche", "keinen", "seinen", "ihren",
            "vielen", "und", "oder", "aber", "nicht", "auch", "der", "die", "das", "herein",
            "zurück", "hinunter",
        ] {
            assert!(
                dict.contains_word_str(word),
                "'{word}' must still be accepted by the spell checker"
            );
        }
    }

    /// Finite verb forms are not nouns. `willst/~~NY` claimed a countable noun
    /// with a plural, so the linter wanted "Willst" in the middle of a sentence.
    #[test]
    fn finite_verb_forms_are_not_nouns() {
        let dict = curated_german_dictionary();

        for word in [
            "willst",
            "abspülst",
            "abstempeltest",
            "brätst",
            "bereithältst",
        ] {
            let metadata = dict
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("'{word}' should be in the German dictionary"));

            assert!(
                !metadata.is_noun(),
                "'{word}' is a finite verb form, not a noun"
            );
        }

        // Words that only look like verb forms must keep their noun reading.
        for word in ["frost", "forst", "oberst", "anarchist", "arabist"] {
            let metadata = dict
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("'{word}' should be in the German dictionary"));

            assert!(
                metadata.is_noun(),
                "'{word}' is a genuine noun and must keep its noun reading"
            );
        }
    }

    /// Directional particles are adverbs, not nouns (`herein/~~NhY` claimed a
    /// countable noun with a plural).
    #[test]
    fn separable_prefixes_are_not_nouns() {
        let dict = curated_german_dictionary();

        for word in [
            "herein",
            "hinauf",
            "herauf",
            "herab",
            "herunter",
            "hinunter",
            "herüber",
            "hinüber",
            "auseinander",
            "empor",
            "voran",
            "vorbei",
            "vorüber",
        ] {
            let metadata = dict
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("'{word}' should be in the German dictionary"));

            assert!(
                !metadata.is_noun(),
                "'{word}' is a directional adverb / separable verb prefix, not a noun"
            );
        }
    }

    fn flagged_words(text: &str) -> Vec<String> {
        let dict = curated_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::Standard, dict.clone());
        let document = Document::new(text, &PlainGerman, &dict);

        linter
            .lint(&document)
            .into_iter()
            .map(|lint| document.get_span_content_str(&lint.span))
            .collect()
    }

    /// Ordinary German prose made almost entirely of function words should not
    /// produce capitalization lints.
    #[test]
    fn function_words_are_not_flagged_in_running_text() {
        let sentences = [
            "Er hat es mit ihr und mir gemacht, aber sie wollte nicht.",
            "Wenn du das nicht willst, dann sag es mir bitte auch noch heute.",
            "Sie ging herein und kam kurz darauf wieder heraus.",
            "Jeder von uns weiß, dass wir zusammen mehr erreichen.",
            "Du spülst ab und stempelst die Karte, während wir warten.",
        ];

        for sentence in sentences {
            let flagged = flagged_words(sentence);
            assert!(
                flagged.is_empty(),
                "no capitalization lint expected in {sentence:?}, got {flagged:?}"
            );
        }
    }
}
