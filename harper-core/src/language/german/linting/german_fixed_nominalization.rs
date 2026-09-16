use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Token, TokenStringExt, document::Document};

/// Fixed phrases whose adjective is nominalized, and therefore capitalized.
///
/// Each entry is (the words that must precede it, the lower-case spelling, the
/// correct one). The preceding words are matched case-insensitively — the phrase
/// may open a sentence — but the nominalized word is matched *exactly*, so a
/// correctly written phrase is never touched.
///
/// This is why the family cannot be Weir rules: Weir matches words
/// case-insensitively, so a rule for `im übrigen` would also match the correct
/// `Im Übrigen` and rewrite it.
const FIXED_NOMINALIZATIONS: &[(&[&str], &str, &str)] = &[
    (&["im"], "übrigen", "Übrigen"),
    (&["im"], "allgemeinen", "Allgemeinen"),
    (&["im"], "wesentlichen", "Wesentlichen"),
    (&["im"], "besonderen", "Besonderen"),
    (&["im"], "einzelnen", "Einzelnen"),
    (&["im"], "folgenden", "Folgenden"),
    (&["im"], "klaren", "Klaren"),
    (&["im"], "nachhinein", "Nachhinein"),
    (&["im"], "geringsten", "Geringsten"),
    (&["im"], "gegenteil", "Gegenteil"),
    (&["des"], "öfteren", "Öfteren"),
    (&["des"], "weiteren", "Weiteren"),
    (&["ohne"], "weiteres", "Weiteres"),
    (&["bei"], "weitem", "Weitem"),
    (&["von"], "neuem", "Neuem"),
    (&["aufs"], "neue", "Neue"),
    (&["zum"], "besten", "Besten"),
    (&["fürs"], "erste", "Erste"),
    (&["auf", "dem"], "laufenden", "Laufenden"),
    (&["auf", "das"], "neue", "Neue"),
    (&["seit", "geraumer"], "zeit", "Zeit"),
];

/// Catches the lower-cased half of a fixed nominalization: *"im übrigen"*,
/// *"des öfteren"*, *"auf dem laufenden"*.
///
/// These are frequent in ordinary German writing and frequently wrong. The spell
/// checker cannot see them — every word is a real word — and
/// `GermanNounCapitalization` will not, because the words carry an adjective
/// reading and are not the head of a noun phrase.
#[derive(Default)]
pub struct GermanFixedNominalization;

impl GermanFixedNominalization {
    /// The correction for the word at `index`, if the phrase before it matches.
    fn correction(tokens: &[&Token], index: usize, document: &Document) -> Option<&'static str> {
        let word: String = document
            .get_span_content(&tokens[index].span)
            .iter()
            .collect();

        for (prefix, lowercase, corrected) in FIXED_NOMINALIZATIONS {
            if word != *lowercase {
                continue;
            }

            if index < prefix.len() {
                continue;
            }

            let matches = prefix.iter().enumerate().all(|(offset, expected)| {
                let token = tokens[index - prefix.len() + offset];
                let actual: String = document
                    .get_span_content(&token.span)
                    .iter()
                    .flat_map(|c| c.to_lowercase())
                    .collect();
                actual == *expected
            });

            if matches && !Self::modifies_a_following_noun(tokens, index, document) {
                return Some(corrected);
            }
        }

        None
    }

    /// Is the candidate an attributive adjective after all, because a noun
    /// follows it?
    ///
    /// The same words go both ways and only the noun tells them apart: *"im
    /// **Folgenden**"* is the nominalization, *"im folgenden **Jahr**"* is an
    /// ordinary adjective. So is *"ohne weiteres **Geld**"* against *"ohne
    /// **Weiteres**"*.
    ///
    /// A capital letter on the next word settles it. One adjective may stand in
    /// between — *"im übrigen deutschen **Sprachraum**"* — but only an adjective,
    /// so that a finite verb (*"im Folgenden **werden** Beispiele genannt"*) does
    /// not hide the nominalization behind the noun after it.
    fn modifies_a_following_noun(tokens: &[&Token], index: usize, document: &Document) -> bool {
        let capitalized = |offset: usize| -> bool {
            tokens
                .get(index + offset)
                .and_then(|t| document.get_span_content(&t.span).first().copied())
                .is_some_and(char::is_uppercase)
        };

        if capitalized(1) {
            return true;
        }

        // Only an adjective may stand between the two. Testing the *ending*
        // instead would catch `werden` — *"im Folgenden werden Beispiele
        // genannt"* — and hide the nominalization behind the finite verb.
        //
        // The adjective reading alone, without also demanding the absence of a
        // verb reading: the dictionary hands out spurious verb readings freely,
        // and `deutschen` carries one.
        let intervening_is_adjective = tokens.get(index + 1).is_some_and(|t| t.kind.is_adjective());

        intervening_is_adjective && capitalized(2)
    }
}

impl Linter for GermanFixedNominalization {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let words: Vec<&Token> = sentence.iter_words().collect();

            for index in 0..words.len() {
                let Some(corrected) = Self::correction(&words, index, document) else {
                    continue;
                };

                let original: String = document
                    .get_span_content(&words[index].span)
                    .iter()
                    .collect();

                lints.push(Lint {
                    span: words[index].span,
                    lint_kind: LintKind::Capitalization,
                    suggestions: vec![Suggestion::ReplaceWith(corrected.chars().collect())],
                    priority: 26,
                    message: format!(
                        "In dieser festen Wendung ist »{original}« ein Substantiv und wird \
                         großgeschrieben: »{corrected}«."
                    ),
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Schreibt die Nominalisierung in festen Wendungen groß (»im Übrigen«, »des Öfteren«)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanFixedNominalization;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn flagged(text: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanFixedNominalization
            .lint(&document)
            .into_iter()
            .map(|lint| document.get_span_content_str(&lint.span))
            .collect()
    }

    #[test]
    fn flags_the_lowercase_half_of_a_fixed_phrase() {
        for (text, word) in [
            ("Das gilt im übrigen für alle.", "übrigen"),
            ("Er kommt des öfteren zu spät.", "öfteren"),
            ("Sie hielten ihn auf dem laufenden.", "laufenden"),
            ("Das gelang ihm ohne weiteres.", "weiteres"),
            ("Er war bei weitem der beste.", "weitem"),
        ] {
            assert_eq!(flagged(text), vec![word.to_string()], "in {text:?}");
        }
    }

    #[test]
    fn leaves_the_correct_spelling_alone() {
        for text in [
            "Das gilt im Übrigen für alle.",
            "Im Übrigen gilt das für alle.",
            "Er kommt des Öfteren zu spät.",
            "Sie hielten ihn auf dem Laufenden.",
        ] {
            assert!(flagged(text).is_empty(), "should not fire on {text:?}");
        }
    }

    #[test]
    fn leaves_the_attributive_reading_alone() {
        for text in [
            "Im folgenden Jahr hatte sie ein Konzert.",
            "Das gilt im allgemeinen Sinn für alle.",
            "Der Automat wirft aus, ohne weiteres Geld zu fordern.",
            "Das gilt im übrigen deutschen Sprachraum genauso.",
            "Die Genese von neuem Wissen ist das Ideal.",
        ] {
            assert!(flagged(text).is_empty(), "should not fire on {text:?}");
        }
    }

    #[test]
    fn a_finite_verb_does_not_hide_the_nominalization() {
        assert_eq!(
            flagged("Im übrigen ist das Haus verkauft."),
            vec!["übrigen".to_string()]
        );
    }

    #[test]
    fn leaves_the_same_words_alone_outside_the_phrase() {
        for text in [
            "Die übrigen Teilnehmer warteten draußen.",
            "Das allgemeinen Wohl dienende Vorhaben scheiterte.",
            "Er hat die Zeit im Blick behalten.",
        ] {
            assert!(flagged(text).is_empty(), "should not fire on {text:?}");
        }
    }
}
