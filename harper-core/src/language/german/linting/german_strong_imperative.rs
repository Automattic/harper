//! *Gebe mir das Buch!*: the imperative of a strong verb raises its *e*,
//! *Gib mir das Buch!*

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::{
        grammar::{noun_phrase::lowercase_of, subjects::subject_pronoun, verbs::raised_stems},
        spell::curated_german_dictionary,
    },
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::Dictionary,
};

/// Reports the imperative of an *e/i* verb built like a regular one: *Gebe
/// mir das!* → *Gib*, *Nehme Platz!* → *Nimm*, *Lese das bitte* → *Lies*,
/// *Vergesse es nicht!* → *Vergiss*.
///
/// The verbs are not listed. A form in *-e* at the start of the sentence is
/// read as such an imperative when the dictionary knows the raised stem as a
/// verb (*gib*), the third person built on it (*gibt*), and the infinitive on
/// the plain stem (*geben*) — and *not* an infinitive on the raised one, which
/// is how *lebe* → *lieb* is told apart: *lieben* is a verb of its own.
/// `add_german_strong_imperatives.py` gives these forms their verb reading.
///
/// The same *-e* form is the first person (*Gebe zu, dass …*, with the
/// pronoun dropped) and the subjunctive (*Gebe Gott, dass …*). So the
/// sentence has to be a request — it ends in *!* or contains *bitte* — and no
/// subject may follow the verb (*Gebe ich dir das?*).
#[derive(Default)]
pub struct GermanStrongImperative;

impl GermanStrongImperative {
    fn is_verb(word: &str) -> bool {
        let chars: Vec<char> = word.chars().collect();
        curated_german_dictionary()
            .get_word_metadata(&chars)
            .is_some_and(|metadata| metadata.is_verb())
    }

    /// The imperative of the strong verb whose regular-looking form `word` is.
    fn strong_imperative(word: &str) -> Option<String> {
        let stem = word.strip_suffix('e')?;
        if stem.chars().count() < 2 || !Self::is_verb(&format!("{stem}en")) {
            return None;
        }
        raised_stems(stem).into_iter().find(|raised| {
            Self::is_verb(raised)
                && Self::is_verb(&format!("{raised}t"))
                && !Self::is_verb(&format!("{raised}en"))
        })
    }

    fn is_request(sentence: &[&Token], document: &Document) -> bool {
        let exclaimed = sentence
            .iter()
            .rev()
            .find(|token| !token.kind.is_whitespace())
            .is_some_and(|token| document.get_span_content(&token.span) == ['!']);
        exclaimed
            || sentence
                .iter()
                .any(|token| token.kind.is_word() && lowercase_of(token, document) == "bitte")
    }

    fn check(words: &[&Token], at: usize, document: &Document) -> Option<Lint> {
        let token = words[at];
        let next = words.get(at + 1)?;
        let word = lowercase_of(token, document);
        if !next.kind.is_word() {
            return None;
        }
        let next_word = lowercase_of(next, document);
        // *Gebe ich dir das?*, *Gebe Gott, dass …*. Not *es*, which is the
        // object far more often: *Vergiss es nicht!*
        if subject_pronoun(&next_word).is_some() || next_word == "gott" {
            return None;
        }
        let fixed = Self::strong_imperative(&word)?;
        Some(Lint {
            span: token.span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case(
                fixed.chars().collect(),
                document.get_span_content(&token.span),
            )],
            message: format!(
                "Der Imperativ von »{}en« hebt das e zu i: »{fixed}«, nicht »{word}«.",
                &word[..word.len() - 1]
            ),
            priority: 31,
        })
    }
}

impl Linter for GermanStrongImperative {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let tokens: Vec<&Token> = sentence.iter().collect();
            if !Self::is_request(&tokens, document) {
                continue;
            }
            let words: Vec<&Token> = tokens
                .iter()
                .copied()
                .filter(|token| !token.kind.is_whitespace())
                .collect();
            // The verb opens the sentence, or follows *bitte* there: *Bitte
            // gebe mir …*, *Bitte, gebe mir …*.
            let mut at = 0;
            if words
                .first()
                .is_some_and(|first| lowercase_of(first, document) == "bitte")
            {
                at = 1;
                if words
                    .get(1)
                    .is_some_and(|t| matches!(t.kind, TokenKind::Punctuation(_)))
                {
                    at = 2;
                }
            }
            if words.get(at).is_some_and(|t| t.kind.is_word())
                && let Some(lint) = Self::check(&words, at, document)
            {
                lints.push(lint);
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Bildet den Imperativ starker Verben mit i: »Gib!«, »Nimm!«, »Lies!« statt »Gebe!«, »Nehme!«, »Lese!«."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanStrongImperative;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn fixes(text: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanStrongImperative
            .lint(&document)
            .into_iter()
            .flat_map(|lint| lint.suggestions)
            .map(|suggestion| suggestion.to_string())
            .collect()
    }

    #[test]
    fn a_regular_looking_imperative_is_reported() {
        for (text, fixed) in [
            ("Gebe mir das Buch!", "Gib"),
            ("Nehme bitte Platz.", "Nimm"),
            ("Lese das bitte noch einmal.", "Lies"),
            ("Vergesse es nicht!", "Vergiss"),
            ("Helfe mir doch mal!", "Hilf"),
            ("Spreche lauter!", "Sprich"),
            ("Esse deinen Teller leer!", "Iss"),
            ("Sehe dir das an!", "Sieh"),
            ("Bitte gebe mir Bescheid.", "gib"),
            ("Bitte, empfehle mich weiter.", "empfiehl"),
            ("Werfe den Ball!", "Wirf"),
        ] {
            assert_eq!(fixes(text), [format!("Replace with: “{fixed}”")], "{text}");
        }
    }

    #[test]
    fn other_verbs_and_other_readings_are_quiet() {
        for text in [
            "Gib mir das Buch!",
            "Nimm bitte Platz.",
            "Lebe deinen Traum!",
            "Mache das bitte!",
            "Gehe nach Hause!",
            "Werde Mitglied!",
            "Stelle dich bitte vor.",
            "Gebe ich dir das Buch?",
            "Gebe zu, dass ich mich geirrt habe.",
            "Lese gerade ein Buch.",
            "Gebe Gott, dass alles gut geht!",
            "Ich gebe dir das Buch!",
        ] {
            assert!(fixes(text).is_empty(), "{text}: {:?}", fixes(text));
        }
    }
}
