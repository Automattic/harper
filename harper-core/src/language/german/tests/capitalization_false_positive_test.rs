//! Regression tests from a false-positive audit of `GermanNounCapitalization`.
//!
//! The 54 archived German Wikipedia articles in `.archive/german-language/`
//! are edited prose — they should produce ~zero capitalization lints, and every
//! one they did produce was a false positive. Cross-checking each flagged word
//! against `aspell -d de` (which accepts a lower-case spelling only when the
//! word really is lower case in German) put 84% of them beyond doubt.
//!
//! Three mechanisms accounted for most of them, and each has a test below:
//! suspended hyphenation, foreign-language glosses, and dictionary entries that
//! carried a corpus-mined noun reading but no adjective/verb one, which made
//! the linter treat them as unambiguous nouns.

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::linting::new_curated_german;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::{LintKind, Linter};

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

    fn assert_clean(text: &str) {
        let flagged = flagged(text);
        assert!(
            flagged.is_empty(),
            "no capitalization lint expected in {text:?}, got {flagged:?}"
        );
    }

    /// German suspends the shared part of coordinated compounds and marks the
    /// gap with a hyphen. Each fragment is correctly lower case.
    #[test]
    fn suspended_hyphenation_is_not_flagged() {
        for text in [
            "Das gilt auf welt-, volks-, stadt- und hauswirtschaftlicher Ebene.",
            "Es gibt Regeln zur Konfliktverhütung und -lösung.",
            "Der Begriff Wirtschaftsform wird auch -weise oder -typ genannt.",
            "Sie fördert Erdöl und -gas.",
        ] {
            assert_clean(text);
        }
    }

    /// A dash used as punctuation is set off by spaces and must not suppress a
    /// real lower-case noun next to it.
    #[test]
    fn a_spaced_dash_still_lets_a_noun_be_flagged() {
        let flagged = flagged("Er kam nach Hause - die stadt war leer.");
        assert!(
            flagged.iter().any(|w| w == "stadt"),
            "'stadt' is a miscapitalized noun and must still be flagged; flagged: {flagged:?}"
        );
    }

    /// Etymology glosses quote the foreign form in lower case after a bare
    /// language name, including down a comma-separated list.
    #[test]
    fn foreign_language_glosses_are_not_flagged() {
        for text in [
            "Gleiches gilt für englisch economy, französisch économie, italienisch economia.",
            "Wie in vielen Sprachen: niederländisch recht, französisch droit, spanisch derecho.",
            "Das Wort althochdeutsch reht, recht, rehd, riht, reth ist alt.",
        ] {
            assert_clean(text);
        }
    }

    /// The gloss rule must not swallow ordinary prose. Only the *uninflected*
    /// language name marks a gloss, and a function word ends it.
    #[test]
    fn the_language_gloss_rule_does_not_leak() {
        for (text, word) in [
            ("Er lernt englisch und geht dann in die stadt.", "stadt"),
            ("Die englische sprache ist weit verbreitet.", "sprache"),
        ] {
            let flagged = flagged(text);
            assert!(
                flagged.iter().any(|w| w == word),
                "{word:?} is a miscapitalized noun in {text:?}; flagged: {flagged:?}"
            );
        }
    }

    /// Present participles used attributively had a noun reading but no
    /// adjective one, so they were flagged wherever they appeared.
    #[test]
    fn attributive_participles_are_not_flagged() {
        for text in [
            "Das sind die folgenden Abschnitte.",
            "Er beschreibt die bestehende Ordnung.",
            "Sie untersucht eine darauf beruhende Annahme.",
            "Das ist eine abweichende Meinung.",
        ] {
            assert_clean(text);
        }
    }

    /// Adjectives and adverbs whose entry was noun-only.
    #[test]
    fn adjectives_and_adverbs_are_not_flagged() {
        for text in [
            "Der Begriff ist heute nicht mehr sehr scharf.",
            "Die Bestimmung bleibt kontrovers.",
            "Es gab eine große mediale Aufmerksamkeit.",
            "Das Ergebnis ist demzufolge eindeutig.",
            "Sie hat das Thema oftmals behandelt.",
        ] {
            assert_clean(text);
        }
    }

    /// None of the above may cost a genuine detection.
    #[test]
    fn genuine_miscapitalized_nouns_are_still_flagged() {
        for (text, word) in [
            ("Der hund spielt im Garten.", "hund"),
            ("Die katze schläft auf dem Sofa.", "katze"),
            ("In der stadt gibt es viele Autos.", "stadt"),
            ("Das kind spielt mit einem Ball.", "kind"),
            ("Die schönheit der Natur ist wunderbar.", "schönheit"),
        ] {
            let flagged = flagged(text);
            assert!(
                flagged.iter().any(|w| w == word),
                "{word:?} must still be flagged in {text:?}; flagged: {flagged:?}"
            );
        }
    }
}
