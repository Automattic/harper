//! German noun-phrase chunking in `GermanNounCapitalization`.
//!
//! English resolves noun/verb/adjective ambiguity with two *contextual* signals
//! that `Document::parse` attaches to every token: `pos_tag`, chosen in context
//! by the trained Brill tagger, and `np_member`, produced by a neural chunker.
//! English linters read those and additionally back off on
//! `DictWordMetadata::is_likely_homograph`.
//!
//! Neither signal is available for German — both models are English-trained —
//! so this linter has to recover noun-phrase structure itself. German makes
//! that tractable: a noun phrase is determiner/preposition, then any number of
//! attributive adjectives, then the head noun. Only the **head** is a candidate
//! for capitalization.
//!
//! The previous one-token-lookback ("is the word to my left an article?")
//! flagged every modifier and missed the head whenever an adjective stood
//! between the two.

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::linting::new_curated_german;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::{LintKind, Linter};

    /// Capitalization lints only — a missing dictionary entry is a coverage
    /// problem, not a chunking one.
    fn flagged(text: &str) -> Vec<String> {
        let dict = curated_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::Standard, dict.clone());
        let document = Document::new(text, &PlainGerman, &dict);
        linter
            .lint(&document)
            .into_iter()
            .filter(|lint| lint.lint_kind == LintKind::Capitalization)
            .map(|lint| document.get_span_content_str(&lint.span))
            .collect()
    }

    /// With several adjectives between the article and the noun, the head is the
    /// one that needs a capital — not the first adjective after the article.
    #[test]
    fn head_of_a_multi_adjective_phrase_is_flagged() {
        let flagged = flagged("Er sieht der große schöne hund an.");

        assert!(
            flagged.iter().any(|w| w == "hund"),
            "'hund' is the head of the phrase and must be flagged; flagged: {flagged:?}"
        );
        for modifier in ["große", "schöne"] {
            assert!(
                !flagged.iter().any(|w| w == modifier),
                "{modifier:?} is attributive and must not be flagged; flagged: {flagged:?}"
            );
        }
    }

    /// Two phrases in one clause; both heads, neither modifier.
    #[test]
    fn every_phrase_head_in_a_sentence_is_flagged() {
        let flagged = flagged("Ich gebe dem kleinen kind einen ball.");

        for head in ["kind", "ball"] {
            assert!(
                flagged.iter().any(|w| w == head),
                "{head:?} is a phrase head and must be flagged; flagged: {flagged:?}"
            );
        }
        assert!(
            !flagged.iter().any(|w| w == "kleinen"),
            "'kleinen' is attributive and must not be flagged; flagged: {flagged:?}"
        );
    }

    /// A capitalized head closes the phrase, so nothing in it is flagged.
    #[test]
    fn correctly_written_phrases_produce_no_lints() {
        for text in [
            "Die deutsche Sprache hat eine komplexe Grammatik.",
            "Das ist eine wesentliche Frage.",
            "Er verbrachte die ganze Zeit im Büro.",
            "Die Stadt beherbergt zahlreiche Museen und Theater.",
        ] {
            let flagged = flagged(text);
            assert!(
                flagged.is_empty(),
                "no capitalization lint expected in {text:?}, got {flagged:?}"
            );
        }
    }

    /// A preposition ends the phrase it does not open: "die Zeit **im** Büro"
    /// is two phrases, so "Zeit" is a head rather than a modifier of "Büro".
    #[test]
    fn a_preposition_closes_the_phrase() {
        let flagged = flagged("Er verbrachte die ganze zeit im büro.");

        for head in ["zeit", "büro"] {
            assert!(
                flagged.iter().any(|w| w == head),
                "{head:?} heads its own phrase and must be flagged; flagged: {flagged:?}"
            );
        }
        assert!(
            !flagged.iter().any(|w| w == "ganze"),
            "'ganze' is attributive and must not be flagged; flagged: {flagged:?}"
        );
    }

    /// The head is the *last* element, and it is capitalized. A lower-case word
    /// after it belongs to the clause, not to the phrase — without that,
    /// "in Munitionsfabriken eingesetzt" runs on and makes the participle the
    /// head of the phrase "in Munitionsfabriken".
    #[test]
    fn a_capitalized_head_closes_the_phrase() {
        for (text, word) in [
            ("Sie wurden in Munitionsfabriken eingesetzt.", "eingesetzt"),
            (
                "Im folgenden Frühjahr befand sich das Regiment dort.",
                "befand",
            ),
            (
                "Bevor die Bucht erreicht wurde, verlor sie den Mast.",
                "erreicht",
            ),
            (
                "Das Regiment nahm an keinen Gefechtseinsätzen teil.",
                "teil",
            ),
        ] {
            let flagged = flagged(text);
            assert!(
                !flagged.iter().any(|w| w == word),
                "{word:?} is outside the phrase in {text:?}; flagged: {flagged:?}"
            );
        }
    }

    /// ...but a lower-case head is still the head, however many adjectives
    /// precede it.
    #[test]
    fn a_lowercase_head_is_still_found() {
        for (text, word) in [
            ("Der hund spielt im Garten.", "hund"),
            ("Er sah die schöne blaue donau.", "donau"),
        ] {
            let flagged = flagged(text);
            assert!(
                flagged.iter().any(|w| w == word),
                "{word:?} heads its phrase in {text:?}; flagged: {flagged:?}"
            );
        }
    }

    /// Outside a noun phrase entirely, a noun/verb homograph stays a verb.
    #[test]
    fn homograph_outside_a_noun_phrase_is_left_alone() {
        let flagged = flagged("Ich sage dir, fang an.");

        assert!(
            !flagged.iter().any(|w| w == "fang"),
            "'fang' is an imperative here and must not be flagged; flagged: {flagged:?}"
        );
    }

    /// ...but the same word at the head of a phrase is the noun.
    #[test]
    fn homograph_as_phrase_head_is_flagged() {
        let flagged = flagged("Der fang ist groß.");

        assert!(
            flagged.iter().any(|w| w == "fang"),
            "'fang' heads the phrase 'der fang' and must be flagged; flagged: {flagged:?}"
        );
    }
}
