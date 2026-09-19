//! Corpus mining tagged a large slice of `dictionary.dict` as nouns regardless
//! of the real part of speech (`beherbergt/~~NhY`, `neue/~~NY`, `überall/~~NhY`).
//!
//! `GermanNounCapitalization` flags a word with an unambiguous noun reading
//! wherever it appears and only context-gates a word that *also* has a verb or
//! adjective reading, so every one of these entries makes an ordinary verb form
//! or attributive adjective get "corrected" mid-sentence. This is the same
//! failure the function-word and adjective-declension fixes addressed; these
//! tests pin the follow-up pass over finite verb forms and common adjectives.
//!
//! `scripts/fix_german_pos_flags.py` is the reproducible source of the retags.

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::linting::new_curated_german;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::Linter;
    use crate::spell::Dictionary;

    /// Finite verb forms that were tagged noun-only now carry a verb reading.
    #[test]
    fn finite_verb_forms_have_a_verb_reading() {
        let dict = curated_german_dictionary();

        for word in [
            "beherbergt",
            "verleiht",
            "dient",
            "enthält",
            "überwacht",
            "bleibt",
            "erstellt",
            "prüft",
            "bestätigt",
            "benötigt",
            "beklagt",
            "bewohnt",
            "bedankt",
            "verließ",
        ] {
            let meta = dict
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("'{word}' should be in the German dictionary"));

            assert!(meta.is_verb(), "'{word}' should have a verb reading");
        }
    }

    /// A subset with no noun homograph at all loses the noun reading outright.
    #[test]
    fn clean_verb_forms_have_no_noun_reading() {
        let dict = curated_german_dictionary();

        for word in [
            "beherbergt",
            "verleiht",
            "bestätigt",
            "benötigt",
            "bedankt",
            "beklagt",
        ] {
            let meta = dict.get_word_metadata_str(word).unwrap();
            assert!(
                !meta.is_noun(),
                "'{word}' is a finite verb form and must not carry a noun reading"
            );
        }
    }

    /// The infinitives behind those forms were noun-only too.
    #[test]
    fn mistagged_infinitives_are_verbs() {
        let dict = curated_german_dictionary();

        for word in ["beherbergen", "überwachen", "prüfen", "unterteilen"] {
            let meta = dict
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("'{word}' should be in the German dictionary"));

            assert!(meta.is_verb(), "'{word}' is an infinitive, not a noun");
        }
    }

    /// Common adjectives whose base entry was noun/verb-only now decline as
    /// adjectives, so the base *and* every generated form carries an adjective
    /// reading. The bare `-e` forms keep their noun reading on purpose ("das
    /// Ganze", "das Wesentliche" are real nouns) — they are noun/adjective
    /// homographs, and `GermanNounCapitalization` tells the two uses apart by
    /// what follows (see `attributive_adjective_before_noun_is_not_flagged`).
    #[test]
    fn common_adjectives_carry_an_adjective_reading() {
        let dict = curated_german_dictionary();

        for word in [
            "ganz",
            "ander",
            "komplex",
            "besonder",
            "modern",
            "weiter",
            "einzeln",
            "korrekt",
            "ganze",
            "andere",
            "komplexe",
            "besondere",
            "moderne",
            "wesentliche",
            "kritische",
            "staatliche",
        ] {
            let meta = dict
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("'{word}' should be in the German dictionary"));

            assert!(
                meta.is_adjective(),
                "'{word}' must carry an adjective reading; without one \
                 GermanNounCapitalization treats it as an unambiguous noun"
            );
        }
    }

    /// Adverbs that were tagged noun-only.
    #[test]
    fn mistagged_adverbs_are_not_nouns() {
        let dict = curated_german_dictionary();

        for word in ["überall", "sofort", "anders", "zeitweise"] {
            let meta = dict
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("'{word}' should be in the German dictionary"));

            assert!(!meta.is_noun(), "'{word}' is an adverb, not a noun");
        }
    }

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

    /// Running prose that used to trip the capitalization linter.
    #[test]
    fn running_text_is_not_over_flagged() {
        for (text, word) in [
            (
                "Die Stadt beherbergt zahlreiche Museen und Theater.",
                "beherbergt",
            ),
            ("Das verleiht ihr ein internationales Flair.", "verleiht"),
            (
                "Der Dienst überwacht die Sitzung und erstellt ein Protokoll.",
                "überwacht",
            ),
            (
                "Der Dienst überwacht die Sitzung und erstellt ein Protokoll.",
                "erstellt",
            ),
            ("Das Werkzeug ist überall im Einsatz.", "überall"),
        ] {
            let flagged = flagged(text);
            assert!(
                !flagged.iter().any(|w| w == word),
                "{word:?} should not be flagged in {text:?}; flagged: {flagged:?}"
            );
        }
    }

    /// A noun/adjective homograph directly before the capitalized head noun is
    /// attributive — leave it alone even though the left context (article,
    /// preposition, ...) licenses a noun phrase.
    #[test]
    fn attributive_adjective_before_noun_is_not_flagged() {
        for (text, word) in [
            ("Das ist eine wesentliche Frage.", "wesentliche"),
            (
                "Berlin ist bekannt für seine vielfältige Gastronomie.",
                "vielfältige",
            ),
            ("Wir haben eine neue Version veröffentlicht.", "neue"),
            ("Er verbrachte die ganze Zeit im Büro.", "ganze"),
            ("Die staatliche Verwaltung reagiert.", "staatliche"),
            ("Das war eine kritische Analyse.", "kritische"),
        ] {
            let flagged = flagged(text);
            assert!(
                !flagged.iter().any(|w| w == word),
                "{word:?} is attributive in {text:?} and must not be flagged; flagged: {flagged:?}"
            );
        }
    }

    /// The same homograph standing as the (nominalized) head of the phrase is
    /// still flagged for capitalization.
    #[test]
    fn nominalized_adjective_is_still_flagged() {
        for (text, word) in [
            ("Er konzentriert sich auf das wesentliche.", "wesentliche"),
            ("Das ist das ganze.", "ganze"),
        ] {
            let flagged = flagged(text);
            assert!(
                flagged.iter().any(|w| w == word),
                "{word:?} is a nominalized noun in {text:?} and should be flagged; flagged: {flagged:?}"
            );
        }
    }
}
