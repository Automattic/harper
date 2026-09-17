//! Where a German adjective may stand, and when it is a noun.
//!
//! [`GermanNounCapitalization`] flags a noun/adjective homograph only in the
//! **head** position of a noun phrase. Everything here is a case where the
//! chunker used to put the head on an adjective — because the phrase was cut
//! short, because the sentence was split at an ordinal, or because a bare
//! preposition was read as a determiner — and reported an ordinary attributive
//! or predicative adjective as a capitalization error.
//!
//! The three grammar facts behind the fixes:
//!
//! 1. A nominalized adjective needs a **determiner** and carries a **declension
//!    ending**: *das Gute*, *im Freien*, *für Deutsche*. The base form after a
//!    bare preposition is predicative — *weiß bis braun*.
//! 2. German compounds are **right-headed**: *Stickstoff* + *tolerant* is an
//!    adjective.
//! 3. `der`/`die`/`das` after a comma is a **relative pronoun**, and what
//!    follows it is a verb-final clause, not a noun phrase.
//!
//! Every case below comes from the archived Wikipedia corpus.

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

    fn assert_quiet(text: &str, word: &str) {
        let lints = flagged(text);
        assert!(
            !lints.iter().any(|l| l == word),
            "'{word}' must not be flagged in {text:?}; flagged: {lints:?}"
        );
    }

    /// A base-form adjective under a bare preposition or numeral is predicative.
    #[test]
    fn predicative_adjectives_are_not_nouns() {
        assert_quiet("Die gefächerte Markzone ist weiß bis braun.", "braun");
        assert_quiet("Die Schriftfarbe wurde von gelb zu weiß geändert.", "gelb");
        assert_quiet("Davon sind vier unbewohnt.", "unbewohnt");
    }

    /// A *declined* adjective is a different matter: "für Deutsche" is a
    /// nominalization even though "für" supplies no determiner, and the rule
    /// above must not swallow it.
    #[test]
    fn declined_adjective_after_a_preposition_is_still_a_noun() {
        let lints = flagged("Dieses Buch ist nur für deutsche.");
        assert!(
            lints.iter().any(|l| l == "deutsche"),
            "'deutsche' is nominalized here and should be flagged; flagged: {lints:?}"
        );
    }

    /// The head may be several tokens away: a contrastive conjunction, a
    /// numeral, an ordinal or a unit can stand between it and the adjective.
    #[test]
    fn the_head_is_reached_across_what_german_allows_inside_a_phrase() {
        assert_quiet("Der milde, aber wenig angenehme Pilz ist da.", "milde");
        assert_quiet("Eine große, 1671 gefertigte Uhr hängt dort.", "große");
        assert_quiet("Der lange 0,9 m breite Gang ist hoch.", "lange");
        assert_quiet(
            "Im Jahre 2025 wurde die ehemalige 84. Oberschule umbenannt.",
            "ehemalige",
        );
    }

    /// An entry the dictionary lists with no part of speech at all carries no
    /// evidence, so it must not end the phrase either.
    #[test]
    fn a_word_without_a_part_of_speech_does_not_end_the_phrase() {
        assert_quiet(
            "Im Tal findet sich eine reiche und diversifizierte Tierwelt.",
            "reiche",
        );
    }

    /// German compounds are right-headed, so a compound ending in an adjective
    /// is an adjective.
    #[test]
    fn compound_ending_in_an_adjective_is_an_adjective() {
        assert_quiet("Die Bakterien sind galleresistent.", "galleresistent");
        assert_quiet(
            "Die Art ist relativ stickstofftolerant.",
            "stickstofftolerant",
        );
    }

    /// An adjective in front of a homographic head still settles the reading:
    /// "purpur" + "braun" is a colour, not a noun.
    #[test]
    fn compound_opening_with_an_adjective_stays_an_adjective() {
        assert_quiet("Die Farbe ist rosaviolett oder purpurbraun.", "purpurbraun");
    }

    /// After a comma, "der" is a relative pronoun: what follows is a clause, and
    /// the first word in it is not the head of a noun phrase.
    #[test]
    fn a_relative_pronoun_does_not_open_a_noun_phrase() {
        assert_quiet(
            "Der SV Rödinghausen, der zuletzt 2019 den Westfalenpokal gewinnen konnte, fehlt.",
            "zuletzt",
        );
        assert_quiet(
            "Es ist ein Beutelsäuger, der verstreut in einigen Gebieten lebt.",
            "verstreut",
        );
        assert_quiet(
            "Am Logo ist eine Zahl anzubringen, die angibt, wie viele Jahre es hält.",
            "angibt",
        );
    }

    /// The head of an ordinary phrase is still found and still flagged.
    #[test]
    fn the_head_of_a_phrase_is_still_flagged() {
        let lints = flagged("Die wesentliche entscheidung ist offen.");
        assert!(
            lints.iter().any(|l| l == "entscheidung"),
            "the head noun must still be flagged; flagged: {lints:?}"
        );
        assert!(
            !lints.iter().any(|l| l == "wesentliche"),
            "the modifier must not be; flagged: {lints:?}"
        );
    }
}
