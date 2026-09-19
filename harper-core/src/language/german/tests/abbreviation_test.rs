//! Abbreviations German writes with a trailing full stop.
//!
//! `ggf.`, `engl.`, `hg.`, `op.`, `var.` are ordinary German and were reported
//! as misspellings: the tokenizer hands the linter the letters without the stop,
//! and `dictionary.dict` had no entry for them. igerman98 does not list them
//! either, so they come from the curated table in
//! `scripts/add_german_abbreviations.py`, which carries the reasoning about
//! which ones are safe to add.
//!
//! Flag `2` is the abbreviation property, and `GermanNounCapitalization` rejects
//! anything carrying it — `hg` must not be "corrected" to `Hg`.

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::linting::new_curated_german;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::Linter;
    use crate::spell::Dictionary;

    fn flagged(text: &str) -> Vec<String> {
        let dict = curated_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::Standard, dict.clone());
        let document = Document::new(text, &PlainGerman, &dict);
        linter
            .lint(&document)
            .into_iter()
            .map(|lint| document.get_span_content_str(&lint.span))
            .collect()
    }

    #[test]
    fn common_abbreviations_are_words() {
        let dict = curated_german_dictionary();

        for word in [
            "ggf", "evtl", "inkl", "bspw", "dt", "engl", "frz", "lat", "hg", "hrsg", "op", "bd",
            "jh", "jr", "var",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is a German abbreviation and should be in the dictionary"
            );
        }
    }

    #[test]
    fn abbreviations_in_running_text_draw_no_lint() {
        for text in [
            "Die Dämpfung des Messkabels und ggf. die Verstärkung sind entscheidend.",
            "Das Werk erschien 1949 (op. 4) in Frankfurt.",
            "Das Buch wurde hg. von einem Team veröffentlicht.",
        ] {
            let lints = flagged(text);
            assert!(
                lints.is_empty(),
                "expected no lints in {text:?}, got {lints:?}"
            );
        }
    }

    /// An abbreviation must never be "corrected" to a capital.
    #[test]
    fn abbreviations_are_not_miscapitalized_nouns() {
        let lints = flagged("Der Band erschien hg. von einem Team im Jahr 1990.");
        assert!(
            !lints.iter().any(|l| l == "hg"),
            "'hg' is an abbreviation, not a lower-case noun; flagged: {lints:?}"
        );
    }
}
