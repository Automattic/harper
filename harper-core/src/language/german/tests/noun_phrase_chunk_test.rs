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

    // --- Erweitertes Attribut -------------------------------------------
    //
    // German may hang a phrase of its own in front of the adjective that
    // modifies the head: "die **in Mitteleuropa** heimische Pflanzenart". The
    // chunker used to stop at the preposition, crown the adjective in front of
    // it, and report a perfectly ordinary attributive as a noun that had lost
    // its capital. It carries on now — but only when the word in front of the
    // preposition is an adjective and the attribute closes on an adjective
    // with a capitalized word behind it, because a preposition otherwise
    // really does end the phrase.

    /// The construction the change is about.
    #[test]
    fn extended_attribute_keeps_the_adjective_a_modifier() {
        let flagged = flagged("Er kennt die einzige in Mitteleuropa heimische Pflanzenart.");

        assert!(
            flagged.is_empty(),
            "the phrase runs on to 'Pflanzenart'; flagged: {flagged:?}"
        );
    }

    /// The attribute may be a bare directional phrase with no noun of its own.
    #[test]
    fn extended_attribute_may_be_directional() {
        let flagged = flagged("Die ganze nach links hinten verlagerte Last drückt auf das Rad.");

        assert!(
            !flagged.iter().any(|w| w == "ganze"),
            "'ganze' is attributive; flagged: {flagged:?}"
        );
    }

    /// A participle is the usual close of one.
    #[test]
    fn extended_attribute_closing_on_a_participle() {
        let flagged = flagged("Das lange von Hand geschriebene Manuskript liegt dort.");

        assert!(
            !flagged.iter().any(|w| w == "lange"),
            "'lange' is attributive; flagged: {flagged:?}"
        );
    }

    /// Two prepositions inside one attribute still close on the same adjective.
    #[test]
    fn extended_attribute_with_two_prepositions() {
        let flagged =
            flagged("Er sah die rote in der Mitte des Raumes auf dem Tisch stehende Vase.");

        assert!(
            !flagged.iter().any(|w| w == "rote"),
            "'rote' is attributive; flagged: {flagged:?}"
        );
    }

    /// An adverb inside the attribute does not end it either.
    #[test]
    fn extended_attribute_with_an_adverb_inside() {
        let flagged =
            flagged("Die hohe in den letzten Jahren stark gestiegene Miete belastet ihn.");

        assert!(
            !flagged.iter().any(|w| w == "hohe"),
            "'hohe' is attributive; flagged: {flagged:?}"
        );
    }

    /// The attribute may govern a determiner of its own.
    #[test]
    fn extended_attribute_with_its_own_determiner() {
        let flagged = flagged("Die grüne für den Winter geeignete Jacke hängt dort.");

        assert!(
            !flagged.iter().any(|w| w == "grüne"),
            "'grüne' is attributive; flagged: {flagged:?}"
        );
    }

    /// A masculine determiner behaves the same way.
    #[test]
    fn extended_attribute_after_a_masculine_determiner() {
        let flagged = flagged("Der kluge mit vielen Büchern ausgestattete Raum gefiel ihm.");

        assert!(
            !flagged.iter().any(|w| w == "kluge"),
            "'kluge' is attributive; flagged: {flagged:?}"
        );
    }

    /// Eight tokens of attribute are still read as one.
    #[test]
    fn extended_attribute_at_full_reach() {
        let flagged =
            flagged("Er sah die rote mit sehr viel Mühe von einem alten Meister gemalte Vase.");

        assert!(
            !flagged.iter().any(|w| w == "rote"),
            "'rote' is attributive; flagged: {flagged:?}"
        );
    }

    /// The head after an attribute is still the head, and still flagged.
    #[test]
    fn head_after_an_extended_attribute_is_flagged() {
        let flagged = flagged("Die junge in Berlin geborene Ärztin behandelt die wunde.");

        assert!(
            flagged.iter().any(|w| w == "wunde"),
            "'wunde' is a head and must be flagged; flagged: {flagged:?}"
        );
        assert!(
            !flagged.iter().any(|w| w == "junge"),
            "'junge' is attributive; flagged: {flagged:?}"
        );
    }

    /// A lower-case head leaves the attribute unrecognised — and that is the
    /// safe direction: the error is reported rather than explained away.
    #[test]
    fn a_lowercase_head_after_an_attribute_is_still_reported() {
        let flagged = flagged("Der hohe in der Stadt stehende turm ist alt.");

        assert!(
            flagged.iter().any(|w| w == "turm"),
            "'turm' is a lower-case head and must be flagged; flagged: {flagged:?}"
        );
    }

    /// The difference that makes the walk safe: after a **noun** the
    /// prepositional phrase is a postmodifier and the phrase is finished.
    #[test]
    fn a_postmodifier_is_not_an_extended_attribute() {
        let flagged = flagged("Er sah die blume in dem großen Garten.");

        assert!(
            flagged.iter().any(|w| w == "blume"),
            "'blume' is the head of its phrase; flagged: {flagged:?}"
        );
    }

    /// Same with a proper name behind the preposition.
    #[test]
    fn a_postmodifier_naming_a_place_is_not_an_attribute() {
        let flagged = flagged("Er sah die blume in Berlin.");

        assert!(
            flagged.iter().any(|w| w == "blume"),
            "'blume' is the head of its phrase; flagged: {flagged:?}"
        );
    }

    /// A contracted preposition ends the phrase as it always did.
    #[test]
    fn a_contraction_still_ends_the_phrase() {
        let flagged = flagged("Die zeit im Büro war lang.");

        assert!(
            flagged.iter().any(|w| w == "zeit"),
            "'zeit' is the head of its phrase; flagged: {flagged:?}"
        );
    }

    /// Two postmodified phrases in one sentence, both heads reported.
    #[test]
    fn two_postmodified_phrases_keep_both_heads() {
        let flagged = flagged("Er kaufte die blume in Berlin und die vase in Rom.");

        for head in ["blume", "vase"] {
            assert!(
                flagged.iter().any(|w| w == head),
                "{head:?} is a head and must be flagged; flagged: {flagged:?}"
            );
        }
    }

    /// An ordinary adjective chain is untouched by any of this.
    #[test]
    fn a_plain_adjective_chain_is_unchanged() {
        let flagged = flagged("Er kannte die schöne alte stadt.");

        assert!(
            flagged.iter().any(|w| w == "stadt"),
            "'stadt' is the head; flagged: {flagged:?}"
        );
        for modifier in ["schöne", "alte"] {
            assert!(
                !flagged.iter().any(|w| w == modifier),
                "{modifier:?} is attributive; flagged: {flagged:?}"
            );
        }
    }

    /// The search for the attribute's close stops at the end of the sentence.
    #[test]
    fn the_attribute_search_stops_at_the_sentence_end() {
        let flagged = flagged("Er kannte die alte stadt. In Berlin gebaute Hallen sind selten.");

        assert!(
            flagged.iter().any(|w| w == "stadt"),
            "'stadt' is the head of the first sentence; flagged: {flagged:?}"
        );
    }
}
