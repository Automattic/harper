use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{CharStringExt, Span, TokenKind, TokenStringExt, document::Document};

/// Words German legitimately writes twice in a row.
///
/// Each is a form that is both an article or pronoun and a relative or
/// personal pronoun, so the second copy opens a new phrase:
///
/// * *die Frauen, die die Schuhe tragen* — relative pronoun, then article.
/// * *weil sie sie mag*, *wenn ihr ihr helft* — subject, then object.
///
/// The price is that *repariert die die Sohle* goes unreported: the doubled
/// article and the relative pronoun are the same two words.
const LEGITIMATE_REPEATS: &[&str] = &["der", "die", "das", "den", "dem", "des", "sie", "ihr"];

/// Reports a word written twice in a row: *Ich ich habe*, *und und*.
///
/// The shared English rule cannot be borrowed: it decides by part of speech,
/// in English, and its message is English. German needs the opposite of its
/// homograph test — the doubled pronouns above are the *only* repeats that are
/// grammatical — so the list is short and explicit.
///
/// Two capitalized copies are left alone: *Bora Bora* and *Duran Duran* are
/// names. A capital on the first copy only counts at the start of a sentence
/// (*Ich ich*); elsewhere it marks a noun beside its verb (*Reden reden*).
#[derive(Default)]
pub struct GermanRepeatedWords;

impl Linter for GermanRepeatedWords {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            let words: Vec<_> = chunk.iter_word_indices().zip(chunk.iter_words()).collect();
            let sentence_initial = |index: usize| {
                chunk[..index]
                    .iter()
                    .rev()
                    .find(|t| !t.kind.is_whitespace())
                    .is_none_or(|t| t.kind.is_sentence_terminator())
            };

            for pair in words.windows(2) {
                let [(idx_a, tok_a), (idx_b, tok_b)] = pair else {
                    continue;
                };

                if !chunk[idx_a + 1..*idx_b]
                    .iter()
                    .all(|t| t.kind.is_whitespace())
                {
                    continue;
                }

                let a = document.get_span_content(&tok_a.span);
                let b = document.get_span_content(&tok_b.span);
                let lower = a.to_lower();
                if lower.as_ref() != b.to_lower().as_ref() {
                    continue;
                }

                let lower: String = lower.iter().collect();
                if LEGITIMATE_REPEATS.contains(&lower.as_str()) {
                    continue;
                }

                let capitalized = |w: &[char]| w.first().is_some_and(|c| c.is_uppercase());
                if capitalized(a) && capitalized(b) {
                    continue;
                }

                // A capital on the first copy alone is a sentence start (*Ich
                // ich*) or a different word: *Reden reden*, *das Lernen
                // lernen*, *einen Weg weg*.
                if capitalized(a) && !sentence_initial(*idx_a) {
                    continue;
                }

                // `kg KG`, `u. U.`, `etc. etc.`: abbreviations and single
                // letters.
                let abbreviated = |w: &[char]| {
                    w.iter().skip(1).any(|c| c.is_uppercase())
                        || !w.iter().all(|c| c.is_alphabetic())
                };
                if a.len() < 2 || abbreviated(a) || abbreviated(b) {
                    continue;
                }

                // *Bufo bufo*, *Vipera aspis aspis*: Latin tautonyms, which the
                // dictionary does not know.
                if !matches!(tok_a.kind, TokenKind::Word(Some(_))) {
                    continue;
                }

                if chunk
                    .get(idx_a.wrapping_sub(1))
                    .is_some_and(|t| t.kind.is_hyphen())
                    || chunk.get(idx_b + 1).is_some_and(|t| t.kind.is_hyphen())
                {
                    continue;
                }

                let word: String = a.iter().collect();
                lints.push(Lint {
                    span: Span::new(tok_a.span.start, tok_b.span.end),
                    lint_kind: LintKind::Repetition,
                    suggestions: vec![Suggestion::ReplaceWith(a.to_vec())],
                    message: format!("»{word}« steht hier zweimal hintereinander."),
                    priority: 63,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Findet Wörter, die versehentlich doppelt geschrieben wurden (»ich ich«, »und und«)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanRepeatedWords;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn lint_count(text: &str) -> usize {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanRepeatedWords.lint(&document).len()
    }

    #[test]
    fn flags_doubled_words() {
        for text in [
            "Ich ich habe meine Schuhe vergessen.",
            "Wir gehen und und kaufen Schuhe.",
            "Er hat hat die Stiefel repariert.",
            "Die Sohle ist ist kaputt.",
            "Ich weiß, dass dass der Schuh drückt.",
            "Der Schuh passt nicht nicht mehr.",
            "Er kauft sich einen einen neuen Schuh.",
            "Wir waren im im Schuhgeschäft.",
            "Sie werden gespeichert, um um Aufmerksamkeit zu erhalten.",
        ] {
            assert_eq!(lint_count(text), 1, "should fire on {text:?}");
        }
    }

    #[test]
    fn leaves_grammatical_repeats_alone() {
        for text in [
            "Die Frauen, die die Schuhe tragen, lachen.",
            "Der Mann, der der Frau hilft, ist nett.",
            "Ich weiß, dass sie sie mag.",
            "Wenn ihr ihr helft, ist sie froh.",
            "Das Kind, dem dem Hund der Ball gehört, spielt.",
            "Wir fliegen nach Bora Bora.",
            "Der Schuh passt, passt aber nicht gut.",
            "Das ist ein Baden-Baden-Ausflug.",
            "Die Erdkröte (Bufo bufo) ist häufig.",
            "Das Lernen lernen ist wichtig.",
            "Er geht einen Weg weg von der Straße.",
            "Die Kosten sind u. U. höher.",
            "Man nimmt 1,3 g/kg KG auf.",
        ] {
            assert_eq!(lint_count(text), 0, "should not fire on {text:?}");
        }
    }
}
