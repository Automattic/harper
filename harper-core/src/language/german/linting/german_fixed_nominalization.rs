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
///
/// **Only phrases where the capital is obligatory belong here.** The 1996
/// reform capitalized this family, but for a handful of them it left the small
/// letter standing as an equal variant — `bei weitem`, `ohne weiteres`, `von
/// neuem`, `aufs neue`, `zum besten` are all still correct as written, and
/// flagging them told writers that correct German was a mistake. They were in
/// this table and are deliberately not any more.
///
/// The line between the two groups is not one a reader can feel, so check a
/// candidate against LanguageTool, which encodes the official rules, before
/// adding it — in both spellings:
///
/// ```bash
/// docker start lt-bench   # erikvl87/languagetool, port 8010
/// curl -s -X POST http://localhost:8010/v2/check -d language=de-DE \
///     --data-urlencode "text=Das gilt im übrigen für alle." |
///     python3 -c 'import json,sys; print([m["rule"]["id"] for m in json.load(sys.stdin)["matches"]])'
/// ```
///
/// A phrase LanguageTool leaves alone in *both* spellings permits both and does
/// not belong in this table.
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
    (&["fürs"], "erste", "Erste"),
    (&["auf", "dem"], "laufenden", "Laufenden"),
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
    /// ordinary adjective. So is *"im einzelnen **Fall**"* against *"im
    /// **Einzelnen**"*.
    ///
    /// A capital letter on the next word settles it. One adjective may stand in
    /// between — *"im übrigen deutschen **Sprachraum**"* — but only an adjective,
    /// so that a finite verb (*"im Folgenden **werden** Beispiele genannt"*) does
    /// not hide the nominalization behind the noun after it.
    ///
    /// And only an adjective that could really be in the *same* noun phrase.
    /// *"im wesentlichen seinen **Höhepunkt**"* and *"im wesentlichen zwei
    /// **Methoden**"* have a capitalized noun two words along and an adjective
    /// reading in between, yet `wesentlichen` is the nominalization in both: the
    /// noun phrase starts at `seinen` and at `zwei`, not before them.
    /// [`opens_its_own_phrase`] is what tells them apart.
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

        if tokens
            .get(index + 1)
            .is_some_and(|t| Self::opens_its_own_phrase(tokens[index], t, document))
        {
            return false;
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

    /// Does `next` start a noun phrase of its own, rather than continue the one
    /// the candidate would be in?
    ///
    /// Two signals, both grammatical rather than a list of words:
    ///
    /// - **A determiner.** German allows one per noun phrase and it comes
    ///   first, so a determiner after the candidate means the candidate is not
    ///   in the same phrase — *"im Wesentlichen **seinen** Höhepunkt"*.
    /// - **A disagreeing ending.** Attributive adjectives in one phrase agree,
    ///   and after `im` or `des` they are all weak: *"im übrigen deutschen
    ///   Sprachraum"* is `-en` twice over. An adjective with a different ending,
    ///   or none — *"im Wesentlichen **zwei** Methoden"* — belongs to something
    ///   else. This is also why the cardinals need no list: `zwei`, `drei` and
    ///   `vier` are uninflected and so can never agree.
    fn opens_its_own_phrase(candidate: &Token, next: &Token, document: &Document) -> bool {
        if next.kind.is_determiner() {
            return true;
        }

        let ending = |token: &Token| {
            let word: String = document.get_span_content(&token.span).iter().collect();
            ["em", "en", "er", "es", "e"]
                .into_iter()
                .find(|suffix| word.ends_with(suffix))
                .unwrap_or("")
        };

        ending(next) != ending(candidate)
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
            ("Im einzelnen sind das drei Punkte.", "einzelnen"),
            ("Fürs erste reicht das.", "erste"),
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
            "Im einzelnen Fall mag das anders sein.",
            "Das gilt im übrigen deutschen Sprachraum genauso.",
        ] {
            assert!(flagged(text).is_empty(), "should not fire on {text:?}");
        }
    }

    /// A capitalized noun two words along does not make the candidate its
    /// attribute: the noun phrase can start in between.
    #[test]
    fn a_new_noun_phrase_does_not_hide_the_nominalization() {
        for (text, word) in [
            (
                "Der Idealismus erreichte im wesentlichen seinen Höhepunkt.",
                "wesentlichen",
            ),
            (
                "Man unterscheidet im wesentlichen zwei Methoden.",
                "wesentlichen",
            ),
            (
                "Das Buch ist im wesentlichen eine Ausarbeitung.",
                "wesentlichen",
            ),
        ] {
            assert_eq!(flagged(text), vec![word.to_string()], "in {text:?}");
        }
    }

    /// The reform left the small letter standing as an equal variant in this
    /// handful, so flagging them calls correct German a mistake.
    ///
    /// Verified against LanguageTool, which leaves every one of these alone in
    /// both spellings; see the note on `FIXED_NOMINALIZATIONS`.
    #[test]
    fn leaves_the_phrases_that_permit_both_spellings_alone() {
        for text in [
            "Er war bei weitem der beste.",
            "Das gelang ihm ohne weiteres.",
            "Damit begann der Kreislauf von neuem.",
            "Sie machten sich aufs neue an die Arbeit.",
            "Er gab eine Geschichte zum besten.",
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
