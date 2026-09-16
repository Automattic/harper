use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Token, TokenKind, TokenStringExt, document::Document};

/// Conjunctions that always open a subordinate clause, and therefore always take
/// a comma in front of them.
///
/// Deliberately a short list. `während`, `wenn`, `als` and `damit` are left out
/// because each is also something else — a preposition, part of `wenn auch`, a
/// comparison, a pronominal adverb — and telling them apart needs more than the
/// word itself.
const SUBORDINATORS: &[&str] = &[
    "weil",
    "obwohl",
    "obgleich",
    "obschon",
    "sodass",
    "falls",
    "sobald",
    "solange",
    "nachdem",
    "bevor",
    "zumal",
    "sofern",
    "wohingegen",
];

/// Words that may stand between the comma and the conjunction, carrying the
/// comma further to the left: *"Er kam, **vor allem** weil es regnete"*.
const FOCUS_PARTICLES: &[&str] = &[
    // focus particles
    "allem",
    "gerade",
    "nur",
    "auch",
    "besonders",
    "eben",
    "schon",
    "erst",
    "insbesondere",
    "vor",
    "selbst",
    "sogar",
    "immer",
    // coordinators: the comma belongs in front of *them*
    "und",
    "oder",
    "aber",
    "sondern",
    "denn",
    "doch",
    // "je nachdem" is a fixed phrase, not a subordinate clause
    "je",
];

/// Modifiers that fuse with a *temporal* conjunction — "noch bevor", "kurz
/// nachdem", "unmittelbar bevor" — where the comma goes in front of the pair.
///
/// They are kept apart from [`FOCUS_PARTICLES`] because they do not generalize:
/// "Das Haus steht noch, obwohl es alt ist" needs its comma, and treating `noch`
/// as a particle everywhere would swallow it.
const TEMPORAL_MODIFIERS: &[&str] = &[
    "noch",
    "kurz",
    "lange",
    "unmittelbar",
    "direkt",
    "gleich",
    "kaum",
    "erst",
    "schon",
];

/// The conjunctions [`TEMPORAL_MODIFIERS`] may attach to.
const TEMPORAL_SUBORDINATORS: &[&str] = &["bevor", "nachdem", "sobald", "solange"];

/// Requires the comma German grammar requires in front of a subordinate clause.
///
/// Comma placement is the most common mistake in written German, and most of it
/// needs a parser. This part does not: these conjunctions open a subordinate
/// clause and nothing else, so a comma belongs in front of every one of them
/// that does not start a sentence.
#[derive(Default)]
pub struct GermanSubordinateComma;

impl GermanSubordinateComma {
    /// Is this the conjunction, rather than a proper name spelled the same?
    ///
    /// A conjunction is lower case mid-sentence. `Weil` with a capital is
    /// Stephan Weil, or the place — the corpus has "im Kabinett Weil III".
    fn is_subordinator(token: &Token, document: &Document) -> bool {
        let chars = document.get_span_content(&token.span);
        if chars.first().is_some_and(|c| c.is_uppercase()) {
            return false;
        }
        let word: String = chars.iter().collect();
        SUBORDINATORS.contains(&word.as_str())
    }

    fn is_focus_particle(token: &Token, document: &Document) -> bool {
        Self::word_in(token, document, FOCUS_PARTICLES)
    }

    fn word_in(token: &Token, document: &Document, set: &[&str]) -> bool {
        let word: String = document
            .get_span_content(&token.span)
            .iter()
            .flat_map(|c| c.to_lowercase())
            .collect();
        set.contains(&word.as_str())
    }
}

impl Linter for GermanSubordinateComma {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let tokens: Vec<&Token> = sentence
                .iter()
                .filter(|t| !t.kind.is_whitespace())
                .collect();

            for (index, token) in tokens.iter().enumerate() {
                if !matches!(token.kind, TokenKind::Word(_))
                    || !Self::is_subordinator(token, document)
                {
                    continue;
                }

                // Opening a sentence or a bracket, there is nothing to separate.
                let Some(previous) = index.checked_sub(1).map(|i| tokens[i]) else {
                    continue;
                };

                // Any punctuation before it already does the separating — a
                // comma, a dash, a colon, an opening bracket.
                if !matches!(previous.kind, TokenKind::Word(_)) {
                    continue;
                }

                // An abbreviation's full stop already separates the clauses:
                // "..., z. B. weil ...". The tokenizer keeps the dot on the
                // token, so the word before is "B.".
                if document
                    .get_span_content(&previous.span)
                    .last()
                    .is_some_and(|c| *c == '.')
                {
                    continue;
                }

                // The comma may belong further left, in front of a particle or a
                // coordinating conjunction that governs the clause.
                if Self::is_focus_particle(previous, document) {
                    continue;
                }

                let conjunction: String = document.get_span_content(&token.span).iter().collect();
                if TEMPORAL_SUBORDINATORS.contains(&conjunction.as_str())
                    && Self::word_in(previous, document, TEMPORAL_MODIFIERS)
                {
                    continue;
                }

                lints.push(Lint {
                    span: previous.span,
                    lint_kind: LintKind::Punctuation,
                    suggestions: vec![Suggestion::InsertAfter(vec![','])],
                    priority: 28,
                    message: format!(
                        "»{conjunction}« leitet einen Nebensatz ein. Davor steht ein Komma."
                    ),
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Setzt das Komma vor einer unterordnenden Konjunktion (»weil«, »obwohl«, »falls«)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanSubordinateComma;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn lint_count(text: &str) -> usize {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanSubordinateComma.lint(&document).len()
    }

    #[test]
    fn requires_the_comma() {
        for text in [
            "Er blieb zu Hause weil er krank war.",
            "Das Haus steht noch obwohl es alt ist.",
            "Wir gehen los sobald der Regen aufhört.",
            "Sie rief an nachdem sie angekommen war.",
        ] {
            assert_eq!(lint_count(text), 1, "should fire on {text:?}");
        }
    }

    #[test]
    fn accepts_the_comma() {
        for text in [
            "Er blieb zu Hause, weil er krank war.",
            "Das Haus steht noch, obwohl es alt ist.",
            "Weil er krank war, blieb er zu Hause.",
            "Er kam (weil es regnete) mit dem Auto.",
            "Er kam, vor allem weil es regnete.",
            "Er blieb zu Hause, und weil er krank war, rief er an.",
            // A temporal modifier fuses with a temporal conjunction.
            "Noch bevor es zum Sturm kam, war alles fertig.",
            "Das Flugzeug verunglückte, kurz nachdem es gestartet war.",
            // "je nachdem" is a fixed phrase, not a clause.
            "Die Städte haben je nachdem einen oder zwei Bürgermeister.",
            // An abbreviation's own full stop already separates them.
            "Angaben sind schwierig, z. B. weil sie eine Unterscheidung treffen.",
            // Capitalized, it is a name: "im Kabinett Weil III".
            "Er war Minister im Kabinett Weil III und später Bevollmächtigter.",
        ] {
            assert_eq!(lint_count(text), 0, "should not fire on {text:?}");
        }
    }
}
