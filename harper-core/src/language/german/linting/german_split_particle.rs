use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Punctuation, Token, TokenKind, TokenStringExt, document::Document};

/// Particles of separable verbs that are only ever written together with the
/// verb when the two stand side by side.
///
/// Left out on purpose:
///
/// * `zusammen`, `wieder`, `fest`, `frei` and the like: *zusammen gearbeitet*
///   ("worked together") and *wieder gekommen* ("came again") are correct
///   wherever the particle keeps its own meaning.
/// * `an`, `auf`, `aus`, `mit`, `vor`, `nach`, `zu`, `ab`: they are
///   prepositions too, and *auf gebrachten Wegen* is a preposition in front of
///   an attribute.
/// * `dazu`, `davon`: pronominal adverbs first — *das kann dazu führen*.
/// * `hinein`: it closes *bis in … hinein*, and *bis ins Jahr hinein gefeiert*
///   is correct.
const PARTICLES: &[&str] = &[
    "zurück", "weg", "vorbei", "heim", "fort", "hinaus", "heraus", "herein", "hinauf", "herauf",
    "hinunter", "herunter", "voraus", "empor",
];

/// Reports a separable particle written apart from the verb right behind it:
/// *zurück gebracht* -> *zurückgebracht*, *weg werfen* -> *wegwerfen*.
///
/// When particle and verb are adjacent — the infinitive and the participle —
/// they form one word. In a main clause the particle moves to the end and is
/// separated anyway (*er bringt es zurück*), which is why the verb has to come
/// *after* the particle here.
///
/// The test is the shape of the second word, not a lookup of the joined one:
/// the compound decomposition accepts nearly any concatenation, `zurückschön`
/// included. So the second word has to be lower case with a verb reading and
/// look like a participle (*ge…t*, *ge…en*) or an infinitive (*-en*, *-ern*,
/// *-eln*).
#[derive(Default)]
pub struct GermanSplitParticle;

impl GermanSplitParticle {
    /// Is `token` a participle, or an infinitive that ends its clause?
    ///
    /// The bare `-en` form is also the finite plural, and an adverbial
    /// particle may stand in front of it in the first position: *darüber
    /// hinaus umfassen …*, *bis ins 16. Jahrhundert hinein entstanden*. At the
    /// end of a clause it can only be the infinitive.
    fn verb_shaped(token: &Token, next: Option<&Token>, document: &Document) -> bool {
        let TokenKind::Word(Some(meta)) = &token.kind else {
            return false;
        };
        // A noun reading is no objection: lower-case infinitives carry the
        // nominalization's (`kommen`, das Kommen).
        if !meta.is_verb() {
            return false;
        }

        let chars = document.get_span_content(&token.span);
        if !chars.iter().all(|c| c.is_lowercase()) {
            return false;
        }
        let word: String = chars.iter().collect();
        let participle = word.starts_with("ge")
            && (word.ends_with('t') || word.ends_with("en"))
            && word.len() > 4;
        let clause_ends = next.is_none_or(|t| {
            matches!(
                t.kind,
                TokenKind::Punctuation(
                    Punctuation::Period
                        | Punctuation::Comma
                        | Punctuation::Question
                        | Punctuation::Bang
                        | Punctuation::Semicolon
                )
            )
        });
        let infinitive = clause_ends && ["en", "ern", "eln"].iter().any(|e| word.ends_with(e));
        participle || infinitive
    }
}

impl GermanSplitParticle {
    /// Does the particle close a circumposition opened earlier in the clause?
    ///
    /// *über die Grenzen hinaus gewirkt*, *darüber hinaus genutzt*, *aus sich
    /// heraus bestehen*, *vom Beobachter weg bewegt*: there the particle
    /// belongs to the preposition, not to the verb, and stays apart.
    fn closes_a_circumposition(before: &[&Token], particle: &str, document: &Document) -> bool {
        let openers: &[&str] = match particle {
            "hinaus" => &["über", "darüber"],
            "heraus" => &["aus", "daraus"],
            "weg" => &["von", "vom", "davon"],
            _ => return false,
        };

        before
            .iter()
            .rev()
            .take_while(|t| !matches!(t.kind, TokenKind::Punctuation(Punctuation::Comma)))
            .filter(|t| t.kind.is_word())
            .any(|t| {
                let word: String = document
                    .get_span_content(&t.span)
                    .iter()
                    .flat_map(|c| c.to_lowercase())
                    .collect();
                openers.contains(&word.as_str())
            })
    }
}

impl Linter for GermanSplitParticle {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let tokens: Vec<&Token> = sentence.iter().collect();

            for index in 0..tokens.len().saturating_sub(2) {
                let (particle, gap, verb) = (tokens[index], tokens[index + 1], tokens[index + 2]);
                if !particle.kind.is_word() || !gap.kind.is_whitespace() {
                    continue;
                }

                let particle_text: String =
                    document.get_span_content(&particle.span).iter().collect();
                if !PARTICLES.contains(&particle_text.as_str())
                    || !Self::verb_shaped(verb, tokens.get(index + 3).copied(), document)
                {
                    continue;
                }

                if Self::closes_a_circumposition(&tokens[..index], &particle_text, document) {
                    continue;
                }

                let verb_text: String = document.get_span_content(&verb.span).iter().collect();
                let joined = format!("{particle_text}{verb_text}");

                lints.push(Lint {
                    span: crate::Span::new(particle.span.start, verb.span.end),
                    lint_kind: LintKind::Spelling,
                    suggestions: vec![Suggestion::ReplaceWith(joined.chars().collect())],
                    message: format!(
                        "»{particle_text}« gehört zum Verb und wird mit ihm zusammengeschrieben: »{joined}«."
                    ),
                    priority: 31,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Schreibt eine trennbare Verbpartikel mit dem folgenden Verb zusammen (»zurückgebracht«)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanSplitParticle;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn flagged(text: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanSplitParticle
            .lint(&document)
            .into_iter()
            .map(|lint| document.get_span_content_str(&lint.span))
            .collect()
    }

    #[test]
    fn joins_the_particle_to_the_verb() {
        for (text, span) in [
            ("Wir haben die Schuhe zurück gebracht.", "zurück gebracht"),
            ("Er hat die alten Stiefel weg geworfen.", "weg geworfen"),
            ("Ich will die Sandalen zurück bringen.", "zurück bringen"),
            ("Sie ist schon heim gegangen.", "heim gegangen"),
            ("Er hat es heraus gefunden.", "heraus gefunden"),
            ("Wir sind hinaus gegangen.", "hinaus gegangen"),
            (
                "Der Schuhmacher hat die Arbeit fort gesetzt.",
                "fort gesetzt",
            ),
            ("Kannst du morgen vorbei kommen?", "vorbei kommen"),
            ("Er ist die Treppe herunter gefallen.", "herunter gefallen"),
            ("Sie wollen die Kartons weg werfen.", "weg werfen"),
            (
                "Sie wurden aus dem Sonnensystem hinaus geschleudert.",
                "hinaus geschleudert",
            ),
            (
                "Ereignisse, die bis 1942 zurück reichten, vergaß er.",
                "zurück reichten",
            ),
        ] {
            assert_eq!(flagged(text), vec![span.to_string()], "in {text:?}");
        }
    }

    #[test]
    fn leaves_correct_text_alone() {
        for text in [
            "Wir haben die Schuhe zurückgebracht.",
            "Er bringt die Schuhe zurück.",
            "Er gab die Schuhe zurück, gebracht hatte sie seine Frau.",
            "Wir haben zusammen gearbeitet.",
            "Der Weg gefiel uns.",
            "Er kam zurück und schaute sich um.",
            "Sie ist weg.",
            "Er ging zurück nach Hause.",
            "Die Zeit ist vorbei.",
            "Wir gehen heim.",
            "Ich will zurück zum Laden.",
            "Darüber hinaus umfassen die Gesetze viele Normen.",
            "Das kann dazu führen, dass die Sohle bricht.",
            "Sie wurde bis ins 16. Jahrhundert hinein gefeiert.",
            "Heraus kommen sie erst im März.",
            "Sie haben weit über die Grenzen Chinas hinaus gewirkt.",
            "Das Konzept wird darüber hinaus genutzt.",
            "Systeme können nicht aus sich heraus bestehen.",
            "Die Galaxien werden vom Beobachter weg bewegt.",
        ] {
            assert!(
                flagged(text).is_empty(),
                "should not fire on {text:?}: {:?}",
                flagged(text)
            );
        }
    }
}
