//! *dass Kriminalität zahlt sich nicht aus*, *dass ich habe eine Arbeit
//! gefunden*: a subordinate clause puts its finite verb at the end.

use crate::{
    Punctuation, Span, Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::{
        grammar::{
            noun_phrase::lowercase_of,
            subjects::{
                irregular_finite_verb, irregular_finite_verb_lemma, subordinate_subject_pronoun,
            },
            verbs::joined_separable_verb,
        },
        linting::german_subordinate_comma::SUBORDINATORS,
    },
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Coordinating conjunctions: a new clause starts behind them.
const COORDINATORS: &[&str] = &["und", "oder", "aber", "sondern", "denn", "sowie"];

/// The words of a subordinate clause, and whether punctuation or the end of
/// the sentence closed it — rather than a conjunction, behind which the
/// clause may go on: *dass ich habe Zeit und Lust*.
struct Clause {
    words: Vec<usize>,
    closed: bool,
}

/// Reports a subordinate clause in main-clause order. German puts the finite
/// verb second in a main clause and last in a subordinate one, and learners
/// carry the first order over: *dass Kriminalität zahlt sich nicht aus* →
/// *sich nicht auszahlt*, *dass ich habe eine neue Arbeit gefunden* → *eine
/// neue Arbeit gefunden habe*.
///
/// The finite verb is not found by parsing. Two shapes give it away:
///
/// * **A separable verb split in two.** A main clause sends the particle to
///   the end and leaves the verb in second place; a subordinate clause joins
///   the two at the end. A particle closing the clause, with a verb earlier
///   in it that the dictionary knows joined to it (*aus* + *zahlt* →
///   *auszahlt*), is the main-clause order. Exactly one word may combine with
///   the particle, because a missing comma runs the clause into the main
///   clause behind it: *Wenn er kommt gehen wir aus*.
/// * **An auxiliary or modal right behind a pronoun subject**, with more of
///   the clause after it: *dass ich bin müde*, *bevor er ist gestorben*,
///   *wenn ich möchte Freunde treffen*. The verb has to agree with the
///   pronoun, the clause has to be closed by punctuation, and two infinitives
///   at its end are the *Ersatzinfinitiv*, which does put the auxiliary first:
///   *dass er hat kommen können*. A form that is also the infinitive
///   (*haben*, *können*) is skipped — *dass sie haben will* is correct.
///
/// The conjunctions are those of `GermanSubordinateComma`, which open a
/// subordinate clause and nothing else, plus *wenn* and the old *daß*.
/// *weil* is left out: with the verb second it is spoken German (*weil ich
/// hab keine Zeit*), and annotators of learner texts leave it standing.
#[derive(Default)]
pub struct GermanSubordinateWordOrder;

impl GermanSubordinateWordOrder {
    fn opens_clause(word: &str) -> bool {
        word != "weil" && (matches!(word, "wenn" | "daß") || SUBORDINATORS.contains(&word))
    }

    fn ends_clause(word: &str) -> bool {
        COORDINATORS.contains(&word) || word == "weil" || Self::opens_clause(word)
    }

    fn is_lowercase(token: &Token, document: &Document) -> bool {
        document
            .get_span_content(&token.span)
            .first()
            .is_some_and(|c| c.is_lowercase())
    }

    /// The clause the conjunction at `at` opens.
    fn clause(tokens: &[&Token], at: usize, document: &Document) -> Clause {
        let mut words = Vec::new();
        for (index, token) in tokens.iter().enumerate().skip(at + 1) {
            match token.kind {
                TokenKind::Word(_) => {
                    if Self::ends_clause(&lowercase_of(token, document)) {
                        return Clause {
                            words,
                            closed: false,
                        };
                    }
                    words.push(index);
                }
                // A dash with space around it sets off an insertion: *dass die
                // Kosten – allein Uunet rechnet mit 60 Millionen – an …*.
                TokenKind::Punctuation(Punctuation::Hyphen)
                    if index > 0 && tokens[index - 1].kind.is_whitespace() =>
                {
                    return Clause {
                        words,
                        closed: false,
                    };
                }
                TokenKind::Punctuation(Punctuation::Hyphen | Punctuation::Apostrophe) => {}
                TokenKind::Space(_) | TokenKind::Newline(_) => {}
                // A number is no particle, but the clause runs on past it:
                // *bei Gebühren von bis zu 3,63 Mark*.
                TokenKind::Number(_) => words.push(index),
                TokenKind::Punctuation(_) => {
                    return Clause {
                        words,
                        closed: true,
                    };
                }
                _ => {
                    return Clause {
                        words,
                        closed: false,
                    };
                }
            }
        }
        Clause {
            words,
            closed: true,
        }
    }

    /// The text from behind `verb_at` up to and including `last`, with
    /// `verb` put at its end: the subordinate order.
    fn moved_to_the_end(
        tokens: &[&Token],
        verb_at: usize,
        last: usize,
        verb: &str,
        document: &Document,
    ) -> (Span<char>, String) {
        let rest = tokens[verb_at + 1..=last]
            .iter()
            .map(|token| document.get_span_content_str(&token.span))
            .collect::<String>();
        let rest = rest.trim();
        let replacement = if rest.is_empty() {
            verb.to_string()
        } else {
            format!("{rest} {verb}")
        };
        (
            Span::new(tokens[verb_at].span.start, tokens[last].span.end),
            replacement,
        )
    }

    fn lint_for(span: Span<char>, replacement: String, message: String) -> Lint {
        Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::ReplaceWith(replacement.chars().collect())],
            message,
            priority: 31,
        }
    }

    /// *dass Kriminalität zahlt sich nicht aus*.
    fn split_particle(tokens: &[&Token], clause: &Clause, document: &Document) -> Option<Lint> {
        let words = &clause.words;
        // A subject, the verb, the particle.
        if words.len() < 3 {
            return None;
        }
        let particle_at = *words.last()?;
        let particle = tokens[particle_at];
        if !Self::is_lowercase(particle, document) {
            return None;
        }
        let particle_text = lowercase_of(particle, document);

        // The subject comes first, so the verb is not the first word.
        let mut verbs = words[1..words.len() - 1].iter().filter_map(|&index| {
            let token = tokens[index];
            if !token.kind.is_verb() || !Self::is_lowercase(token, document) {
                return None;
            }
            let joined = joined_separable_verb(&particle_text, &lowercase_of(token, document))?;
            Some((index, joined))
        });
        let (verb_at, joined) = verbs.next()?;
        if verbs.next().is_some() {
            return None;
        }

        // The particle itself moves with the verb, so the rest of the clause
        // runs up to the word in front of it.
        let before_particle = words[words.len() - 2];
        let replacement = if before_particle == verb_at {
            joined.clone()
        } else {
            Self::moved_to_the_end(tokens, verb_at, before_particle, &joined, document).1
        };
        let span = Span::new(tokens[verb_at].span.start, particle.span.end);
        Some(Self::lint_for(
            span,
            replacement,
            format!(
                "Im Nebensatz steht das Verb am Ende und mit seiner Partikel zusammen: »{joined}«."
            ),
        ))
    }

    /// *dass ich habe eine neue Arbeit gefunden*.
    fn early_auxiliary(tokens: &[&Token], clause: &Clause, document: &Document) -> Option<Lint> {
        let words = &clause.words;
        if !clause.closed || words.len() < 3 {
            return None;
        }
        let subject = subordinate_subject_pronoun(&lowercase_of(tokens[words[0]], document))?;
        let verb = lowercase_of(tokens[words[1]], document);
        let lemma = irregular_finite_verb_lemma(&verb)?;
        if !subject.agrees_with(&irregular_finite_verb(&verb)?) {
            return None;
        }
        // *haben*, *können* are the infinitive as well, and a finite verb at
        // the end makes them that: *dass sie haben will*. With a noun or an
        // adjective at the end, the form is the finite plural: *obwohl wir
        // haben wenig Geld*.
        let last = *words.last()?;
        if verb == lemma && tokens[last].kind.is_verb() {
            return None;
        }
        // *dass er hat kommen können*: the auxiliary in front of two
        // infinitives is the standard order.
        let ends_in_en = |index: usize| {
            Self::is_lowercase(tokens[index], document)
                && lowercase_of(tokens[index], document).ends_with("en")
        };
        if ends_in_en(last) && ends_in_en(words[words.len() - 2]) {
            return None;
        }
        let verb_text = document.get_span_content_str(&tokens[words[1]].span);
        let (span, replacement) =
            Self::moved_to_the_end(tokens, words[1], last, &verb_text, document);
        Some(Self::lint_for(
            span,
            replacement,
            format!("Im Nebensatz steht das konjugierte Verb am Ende: »… {verb_text}«."),
        ))
    }
}

impl Linter for GermanSubordinateWordOrder {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        for sentence in document.iter_sentences() {
            let tokens: Vec<&Token> = sentence.iter().collect();
            for (at, token) in tokens.iter().enumerate() {
                if !token.kind.is_word() || !Self::opens_clause(&lowercase_of(token, document)) {
                    continue;
                }
                let clause = Self::clause(&tokens, at, document);
                if let Some(lint) = Self::split_particle(&tokens, &clause, document)
                    .or_else(|| Self::early_auxiliary(&tokens, &clause, document))
                {
                    lints.push(lint);
                }
            }
        }
        lints
    }

    fn description(&self) -> &str {
        "Stellt im Nebensatz das konjugierte Verb ans Ende: »dass es sich nicht auszahlt« statt »dass es zahlt sich nicht aus«, »dass ich müde bin« statt »dass ich bin müde«."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanSubordinateWordOrder;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn fixes(text: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanSubordinateWordOrder
            .lint(&document)
            .into_iter()
            .flat_map(|lint| lint.suggestions)
            .map(|suggestion| suggestion.to_string())
            .collect()
    }

    #[test]
    fn a_split_verb_in_a_subordinate_clause_is_reported() {
        for (text, fixed) in [
            (
                "Ich bin der Meinung, dass Kriminalität zahlt sich nicht aus.",
                "sich nicht auszahlt",
            ),
            (
                "Ich meine, daß Kriminalität zahlt sich nicht aus.",
                "sich nicht auszahlt",
            ),
            ("Er sagt, dass er kommt morgen an.", "morgen ankommt"),
            ("Ich weiß nicht, ob sie hört zu.", "zuhört"),
            (
                "Wenn der Zug fährt pünktlich ab, sind wir um acht da.",
                "pünktlich abfährt",
            ),
            (
                "Ich hoffe, dass ich lerne ihn bald kennen.",
                "ihn bald kennenlerne",
            ),
            (
                "Sobald das Kino fängt um acht an, essen wir.",
                "um acht anfängt",
            ),
            (
                "Ich glaube, dass er nimmt am Kurs teil.",
                "am Kurs teilnimmt",
            ),
            (
                "Obwohl sie ruft mich jeden Tag an, vermisse ich sie.",
                "mich jeden Tag anruft",
            ),
        ] {
            assert_eq!(fixes(text), [format!("Replace with: “{fixed}”")], "{text}");
        }
    }

    #[test]
    fn an_auxiliary_behind_the_pronoun_is_reported() {
        for (text, fixed) in [
            (
                "Ich freue mich, dass ich habe eine neue Arbeit gefunden.",
                "eine neue Arbeit gefunden habe",
            ),
            ("Ich glaube, dass ich bin krank.", "krank bin"),
            (
                "Bevor er ist gestorben, hat er geschrieben.",
                "gestorben ist",
            ),
            (
                "Ich hoffe, dass ich kann fast alles verstehen.",
                "fast alles verstehen kann",
            ),
            (
                "Wenn ich möchte mit Freunden treffen, rufe ich an.",
                "mit Freunden treffen möchte",
            ),
            ("Sie fragt, ob du hast Zeit.", "Zeit hast"),
            ("Er sagt, dass es wird bald regnen.", "bald regnen wird"),
            (
                "Obwohl wir haben wenig Geld, fahren wir weg.",
                "wenig Geld haben",
            ),
            (
                "Er erzählt, dass er lädt seine Freunde ein.",
                "seine Freunde einlädt",
            ),
        ] {
            assert_eq!(fixes(text), [format!("Replace with: “{fixed}”")], "{text}");
        }
    }

    #[test]
    fn verb_final_clauses_and_main_clauses_are_quiet() {
        for text in [
            "Ich bin der Meinung, dass sich Kriminalität nicht auszahlt.",
            "Er sagt, dass er morgen ankommt.",
            "Der Zug fährt pünktlich ab.",
            "Er kommt morgen an, weil er Urlaub hat.",
            "Ich weiß, dass er kommt, und sie ruft mich an.",
            "Wenn er kommt gehen wir aus.",
            "Dass er kommt, freut mich.",
            "Sie fragt, ob er mitkommt.",
            "Ich glaube, dass er Angst vor ihr hat.",
            "Ich weiß, dass die Kosten - allein Uunet rechnet mit 60 Millionen - steigen.",
            "Ich weiß, wenn er warten musste - bei Gebühren von bis zu 3 Mark.",
            "Ich weiß, dass er in Berlin wohnt und dort arbeitet.",
            "Ich freue mich, dass ich eine neue Arbeit gefunden habe.",
            "Ich weiß, dass er hat kommen können.",
            "Ich glaube, dass sie haben will, was sie sieht.",
            "Ich frage, ob sie will oder nicht.",
            "Ich weiß, dass wir können das machen wollen.",
            "Ich glaube, dass sie haben möchte, was er hat.",
            "Ich weiß, dass ich bin.",
            "Ich komme nicht, weil ich habe keine Zeit.",
            "Ich weiß, dass er hat Zeit und Lust.",
        ] {
            assert!(fixes(text).is_empty(), "{text}: {:?}", fixes(text));
        }
    }
}
