//! *Warum stirbt die Frau.*: a question that ends in a full stop.
//!
//! A direct question is set apart by its word order, not only by its mark:
//! the interrogative opens the sentence and the finite verb follows it
//! directly (*Warum stirbt die Frau?*). The same word order in a statement is
//! rare, which is what lets a full stop behind it be read as a slip.

use crate::{
    Punctuation, Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::grammar::noun_phrase::lowercase_of,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// The words that open a direct question and are followed by the finite verb.
const INTERROGATIVES: &[&str] = &[
    "wer", "wen", "wem", "wessen", "was", "wo", "wohin", "woher", "wann", "warum", "wieso",
    "weshalb", "weswegen", "wozu", "wofür", "wodurch", "womit", "wonach", "worauf", "woran",
    "worin", "worüber", "wovon", "wie",
];

/// *Welche Frau stirbt?*, *Welchen Weg nimmt er?*: the interrogative is an
/// article, and the noun it belongs to stands between it and the verb.
const INTERROGATIVE_ARTICLES: &[&str] = &["welche", "welcher", "welches", "welchen", "welchem"];

/// *Wie lange dauert das?*, *Wie oft kommt er?*: the words that may follow *wie*
/// and still leave the verb next.
const AFTER_WIE: &[&str] = &[
    "lange", "oft", "alt", "groß", "weit", "schnell", "hoch", "spät", "teuer", "häufig", "stark",
    "schwer", "lang", "früh",
];

/// *Wie viel kostet das?*, *Wie viele Menschen leben hier?*: a quantity, which
/// may have a noun of its own before the verb.
const QUANTITIES: &[&str] = &["viel", "viele", "vielen"];

/// Reports a question that ends in a full stop and offers the question mark.
///
/// Only the clearest shape is reported, and only when nothing else in the
/// sentence could make it a statement:
///
/// * the sentence opens with an interrogative, optionally *wie* and a word
///   like *lange* or *viele*, or *welche* and its noun;
/// * a finite verb follows. A participle (*Wie gesagt.*), a pronoun (*Wie man
///   sieht.*) or an adverb (*Wie immer.*) does not;
/// * it has at least three words, so *Wer weiß.* is left alone;
/// * it contains no comma, colon or semicolon. *Was bleibt, ist die Erinnerung.*
///   and *Wer wagt, gewinnt.* have the same opening and are statements. A
///   question with a subordinate clause (*Warum stirbt die Frau, die gestern
///   kam?*) is missed rather than risked.
///
/// A sentence that ends in `?`, `!` or an ellipsis is not touched.
#[derive(Default)]
pub struct GermanQuestionMark;

impl GermanQuestionMark {
    /// Does `token` read as a finite verb?
    ///
    /// A form that is also a participle counts only when `allow_participle`: after
    /// *wie* it is the fixed phrase (*Wie erwartet kam er nicht*), after any other
    /// interrogative it is the verb (*Wieso funktioniert der Plan nicht*, where
    /// *funktioniert* is third person and participle at once).
    fn is_verb_form(token: &Token, document: &Document, allow_participle: bool) -> bool {
        // *einen* is a determiner before it is the infinitive of *einen*, *schon* an
        // adverb before the imperative of *schonen*: *Wer einen guten Schnitt will,
        // …*, *Wie schon 1931 wurde …* are statements.
        if !token.kind.is_verb()
            || token.kind.is_determiner()
            || token.kind.is_pronoun()
            || token.kind.is_adverb()
            || token.kind.is_conjunction()
            || token.kind.is_preposition()
            || !document
                .get_span_content(&token.span)
                .first()
                .is_some_and(|c| c.is_lowercase())
        {
            return false;
        }
        allow_participle
            || !matches!(
                &token.kind,
                TokenKind::Word(Some(metadata)) if metadata.is_verb_past_participle_form()
            )
    }

    /// A quotation mark of any kind. German sets „low“ marks, which the tokenizer
    /// does not know as punctuation, so the characters decide.
    fn is_quote_mark(token: &Token, document: &Document) -> bool {
        matches!(token.kind, TokenKind::Punctuation(Punctuation::Quote(_)))
            || (!token.kind.is_word()
                && !document.get_span_content(&token.span).is_empty()
                && document
                    .get_span_content(&token.span)
                    .iter()
                    .all(|c| "„“”\"‚‘’'»«›‹".contains(*c)))
    }

    /// Is the sentence a plain question in the shape this rule knows, and is it
    /// ended by a full stop? Returns the full stop.
    fn full_stop_of_question<'a>(words: &[&'a Token], document: &Document) -> Option<&'a Token> {
        // *„Warum stirbt die Frau.“*: marks that close the sentence's quote or
        // bracket follow the full stop, and marks that open it come first.
        let start = words.iter().position(|t| {
            !Self::is_quote_mark(t, document)
                && t.kind != TokenKind::Punctuation(Punctuation::OpenRound)
        })?;
        let body = &words[start..];
        let end = body.iter().rposition(|t| {
            !Self::is_quote_mark(t, document)
                && !matches!(
                    t.kind,
                    TokenKind::Punctuation(Punctuation::CloseRound | Punctuation::Apostrophe)
                )
        })?;
        let stop = body[end];
        if !matches!(stop.kind, TokenKind::Punctuation(Punctuation::Period)) {
            return None;
        }

        let clause = &body[..end];
        // Nothing but words, numbers and the quotes of the sentence itself: no
        // comma, colon, further full stop, question mark and so on.
        let plain = clause.iter().all(|t| {
            t.kind.is_word()
                || matches!(t.kind, TokenKind::Number(_))
                || Self::is_quote_mark(t, document)
                || matches!(t.kind, TokenKind::Punctuation(Punctuation::Hyphen))
        });
        let spoken: Vec<&Token> = clause
            .iter()
            .copied()
            .filter(|t| t.kind.is_word() || matches!(t.kind, TokenKind::Number(_)))
            .collect();
        if !plain || spoken.len() < 3 {
            return None;
        }

        let first = lowercase_of(spoken[0], document);
        let second = lowercase_of(spoken[1], document);
        let capitalized = |token: &Token| {
            document
                .get_span_content(&token.span)
                .first()
                .is_some_and(|c| c.is_uppercase())
        };
        let verb_at = if INTERROGATIVES.contains(&first.as_str()) {
            if first == "wie" && AFTER_WIE.contains(&second.as_str()) {
                2
            } else if first == "wie" && QUANTITIES.contains(&second.as_str()) {
                // *Wie viele Menschen leben hier?*, but *Wie viel kostet das?*
                if spoken.get(2).is_some_and(|t| capitalized(t)) {
                    3
                } else {
                    2
                }
            } else {
                1
            }
        } else if INTERROGATIVE_ARTICLES.contains(&first.as_str()) {
            // The noun comes before the verb, capitalized.
            if !capitalized(spoken[1]) {
                return None;
            }
            2
        } else {
            return None;
        };

        let verb = spoken.get(verb_at)?;
        // Only a bare *wie* and its verb can be the start of a fixed phrase.
        let allow_participle = !(first == "wie" && verb_at == 1);
        Self::is_verb_form(verb, document, allow_participle).then_some(stop)
    }
}

impl Linter for GermanQuestionMark {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let words: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();
            if let Some(stop) = Self::full_stop_of_question(&words, document) {
                lints.push(Lint {
                    span: stop.span,
                    lint_kind: LintKind::Punctuation,
                    suggestions: vec![Suggestion::ReplaceWith(vec!['?'])],
                    message: "Ein Fragesatz endet mit einem Fragezeichen.".to_string(),
                    priority: 31,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Ersetzt den Punkt am Ende einer Frage durch ein Fragezeichen: »Warum stirbt die Frau?«."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanQuestionMark;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::{Lint, Linter};

    fn lints(text: &str) -> Vec<Lint> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanQuestionMark.lint(&document)
    }

    /// The text with every suggestion applied.
    fn fixed(text: &str) -> String {
        let mut chars: Vec<char> = text.chars().collect();
        let mut lints = lints(text);
        lints.sort_by_key(|lint| std::cmp::Reverse(lint.span.start));
        for lint in lints {
            for suggestion in lint.suggestions.iter().take(1) {
                suggestion.apply(lint.span, &mut chars);
            }
        }
        chars.into_iter().collect()
    }

    #[test]
    fn a_question_ending_in_a_full_stop_gets_a_question_mark() {
        for (text, expected) in [
            ("Warum stirbt die Frau.", "Warum stirbt die Frau?"),
            ("Wann kommt der Zug an.", "Wann kommt der Zug an?"),
            ("Wer hat das Buch gelesen.", "Wer hat das Buch gelesen?"),
            ("Wie geht es dir heute.", "Wie geht es dir heute?"),
            ("Was ist das für ein Lärm.", "Was ist das für ein Lärm?"),
            ("Wo wohnt ihr jetzt.", "Wo wohnt ihr jetzt?"),
            (
                "Wie lange dauert das Verfahren.",
                "Wie lange dauert das Verfahren?",
            ),
            (
                "Wie viele Menschen leben hier.",
                "Wie viele Menschen leben hier?",
            ),
            (
                "Welche Frau stirbt im Stück.",
                "Welche Frau stirbt im Stück?",
            ),
            ("Welchen Weg nimmt er.", "Welchen Weg nimmt er?"),
            (
                "Wozu brauchen wir das alles.",
                "Wozu brauchen wir das alles?",
            ),
            (
                "Wieso funktioniert der Plan nicht.",
                "Wieso funktioniert der Plan nicht?",
            ),
        ] {
            assert_eq!(fixed(text), expected, "{text}");
        }
    }

    #[test]
    fn the_quotes_around_the_question_are_kept() {
        assert_eq!(
            fixed("„Warum stirbt die Frau.“"),
            "„Warum stirbt die Frau?“"
        );
        assert_eq!(
            fixed("\"Warum stirbt die Frau.\""),
            "\"Warum stirbt die Frau?\""
        );
    }

    #[test]
    fn a_question_inside_a_text_is_found_by_its_own_sentence() {
        assert_eq!(
            fixed("Es war spät. Warum stirbt die Frau. Niemand wusste es."),
            "Es war spät. Warum stirbt die Frau? Niemand wusste es."
        );
    }

    #[test]
    fn a_question_with_its_mark_is_quiet() {
        for text in [
            "Warum stirbt die Frau?",
            "„Warum stirbt die Frau?“",
            "Wie lange dauert das Verfahren?",
            "Warum stirbt die Frau!",
            "Warum stirbt die Frau...",
            "Warum stirbt die Frau",
        ] {
            assert!(lints(text).is_empty(), "{text}");
        }
    }

    #[test]
    fn statements_that_open_like_a_question_are_quiet() {
        for text in [
            // The clause is a subject: a comma follows and then the main clause.
            "Was bleibt, ist die Erinnerung.",
            "Wer wagt, gewinnt.",
            "Wer rastet, der rostet.",
            "Wie die Zeit vergeht, merkt man erst später.",
            // A fixed phrase, not a question.
            "Wie gesagt, das war nicht so gemeint.",
            "Wie immer kam er zu spät nach Hause.",
            "Wie man sieht, ist alles in Ordnung.",
            "Wie dem auch sei, wir machen weiter.",
            "Wie erwartet kam er nicht zur Feier.",
            // The verb is last: a subordinate clause that stands alone in a text.
            "Wann er kommt, weiß ich nicht.",
            "Warum die Frau starb, ist unklar.",
            // The second word has a verb reading but is a determiner or an adverb.
            "Wer einen guten Schnitt will darf keine Discounterpreise erwarten.",
            "Wie schon 1931 wurde Garrett Fort in den Credits genannt.",
            "Wie alle Segler jagen Haussegler in der Luft nach Insekten.",
            "Wer sich was gutes tun will fährt dran vorbei.",
            // Too short to tell.
            "Wer weiß.",
            "Was nun.",
            // An ordinary statement.
            "Die Frau stirbt am Ende.",
            "Er fragte, warum die Frau stirbt.",
            "Ich weiß nicht, wo er wohnt.",
        ] {
            assert!(lints(text).is_empty(), "{text}");
        }
    }

    #[test]
    fn only_the_full_stop_is_replaced() {
        let found = lints("Warum stirbt die Frau.");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].span.start, 21);
        assert_eq!(found[0].span.end, 22);
    }
}
