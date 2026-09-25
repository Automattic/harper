//! German capitalizes its nouns, so a lower-case entry whose **capitalized**
//! form igerman98 does not list is not a noun — whatever corpus mining tagged
//! it. `allenfalls`, `gleichwohl`, `wenngleich`, `mithin`, `desto`, `derart`,
//! `hierdurch`, `diejenige` and tens of thousands of finite verb forms all
//! carried `~~NhY` or `~~NXh`, and `GermanNounCapitalization` flags anything
//! with an unambiguous noun reading.
//!
//! The archived Wikipedia corpus barely shows this: it is biographies and
//! places, where such words hardly occur. On a corpus of abstract German prose —
//! grammar, law, philosophy, mathematics — taking the reading off the entries
//! igerman98 can vouch for removed 1001 of 1574 capitalization lints. `harper-core/src/language/german/scripts/fetch_german_corpus.py` is what fetches that text.
//!
//! Removing `N` is not enough: the noun-plural affixes `X`, `Y`, `a`, `b` and
//! `0` each carry a plural-noun reading of their own.
//! `harper-core/src/language/german/scripts/strip_german_noun_readings.py` removes those too, but only where
//! every hunspell-known form they build survives elsewhere — and where one does
//! not, it hands the job to the verb affix that builds the same strings, so
//! `geh/~~Xh` becomes `geh/~~fh` and still makes `gehe`.

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::linting::new_curated_german;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::Linter;
    use crate::spell::Dictionary;

    /// None of these is a noun in German, and none should read as one.
    #[test]
    fn function_words_are_not_nouns() {
        let dict = curated_german_dictionary();

        for word in [
            "allenfalls",
            "gleichwohl",
            "wenngleich",
            "mithin",
            "desto",
            "derart",
            "hierdurch",
            "jedenfalls",
            "diejenige",
            "dasjenige",
        ] {
            let meta = dict
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("'{word}' should still be a word"));
            assert!(
                !meta.is_noun(),
                "'{word}' is not a noun in German and must not read as one"
            );
        }
    }

    /// Finite verb forms likewise.
    #[test]
    fn finite_verb_forms_are_not_nouns() {
        let dict = curated_german_dictionary();

        for word in ["baut", "braucht", "fragt", "stützt", "birgt", "gehe"] {
            let meta = dict
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("'{word}' should still be a word"));
            assert!(
                !meta.is_noun(),
                "'{word}' is a finite verb form and must not read as a noun"
            );
        }
    }

    /// The forms those affixes were generating are still words. `geh/~~Xh` was
    /// the only source of `gehe`, so the affix moved to `f` rather than going.
    #[test]
    fn the_forms_the_affixes_built_survive() {
        let dict = curated_german_dictionary();

        for word in ["gehe", "bedachten", "gehst", "bedachte"] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is a real German form and must not be lost with the flag"
            );
        }
    }

    /// Words that really are nouns keep their reading, however they are stored.
    /// A slice of `dictionary.dict` holds capitalized nouns in lower case.
    ///
    /// The oracle is igerman98, so its gaps are inherited: it has no `Mithilfe`,
    /// and `mithilfe` therefore lost a noun reading it is entitled to. That is
    /// the price of having an oracle at all, and it is the right way round --
    /// the word is still spelled correctly, it just no longer demands a capital
    /// in `mithilfe der Methode`, where it is a preposition anyway.
    #[test]
    fn real_nouns_keep_their_reading() {
        let dict = curated_german_dictionary();

        for word in ["Haus", "Frage", "Entscheidung", "Verhältnis", "Bedürfnis"] {
            let meta = dict
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("'{word}' should be a word"));
            assert!(meta.is_noun(), "'{word}' is a noun and must read as one");
        }
    }

    /// End to end: a paragraph of abstract prose built from the words that used
    /// to be flagged.
    #[test]
    fn abstract_prose_draws_no_capitalization_lint() {
        let dict = curated_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::default(), dict);

        let source = "Die Untersuchung stützt sich allenfalls auf Vermutungen. \
                      Gleichwohl verbleibt ein Rest, der sich mithin nicht auflösen lässt. \
                      Wer derart fragt, braucht keine Antwort. \
                      Das Gericht baut hierdurch eine Hürde auf, die jedenfalls diejenige \
                      Partei trifft. Je genauer man hinsieht, desto mehr Fragen birgt der Fall.";
        let document = Document::new(source, &PlainGerman, &curated_german_dictionary());

        let flagged: Vec<_> = linter
            .lint(&document)
            .into_iter()
            .filter(|lint| lint.lint_kind == crate::linting::LintKind::Capitalization)
            .collect();
        assert!(
            flagged.is_empty(),
            "none of these words is a noun, got {flagged:?}"
        );
    }
}
