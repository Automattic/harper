use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Punctuation, Token, TokenKind, TokenStringExt, document::Document};

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
];

/// Coordinators, where the comma question is settled rather than moved.
///
/// *»…, und weil er krank war, rief er an«* needs its comma in front of `und`,
/// and *»Sie unterscheiden sich von Komposita … und weil sie den Wortakzent
/// verlieren«* needs none at all — either way the conjunction is not what the
/// comma would separate. So the walk **stops** here and accepts, where at a
/// particle it carries on. Treating these as particles to walk through is what
/// made the first version of this check report five clauses that were already
/// correct.
const CLAUSE_COORDINATORS: &[&str] = &["und", "oder", "aber", "sondern", "denn", "doch"];

/// `je nachdem` is a fixed phrase — "depending" — and not a subordinate clause
/// at all, so the question of a comma does not arise. It sat in
/// [`FOCUS_PARTICLES`] before, which worked only while the check looked exactly
/// one token to the left: walking further finds the verb in "Die Städte haben je
/// nachdem einen …" and asks for a comma again.
const FIXED_PHRASES: &[(&str, &str)] = &[("je", "nachdem")];

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

    /// Does a comma already stand to the left, with only particles in between?
    ///
    /// The comma belongs in front of the material that governs the clause, not
    /// in front of the conjunction: *», wohl weil man…«*, *», teils weil es…«*,
    /// *», einfach weil der Berg da ist«*, *», vermutlich weil…«*, *», etwa
    /// weil er…«*. Looking one token to the left reports every one of those as a
    /// missing comma — eleven of the thirteen this rule produced on a corpus of
    /// published German prose.
    ///
    /// Walking instead of listing is the point. [`FOCUS_PARTICLES`] had the
    /// right idea and could only ever hold the particles somebody thought of;
    /// what actually licenses the comma being further left is that everything
    /// between it and the conjunction modifies the clause rather than being part
    /// of one. A verb or a noun in between means a clause of its own, and the
    /// comma really is missing.
    fn comma_is_further_left(tokens: &[&Token], index: usize, document: &Document) -> bool {
        let mut cursor = index;
        while let Some(previous) = cursor.checked_sub(1).map(|i| tokens[i]) {
            if !matches!(previous.kind, TokenKind::Word(_)) {
                // Punctuation: a comma, dash, colon or bracket separates the
                // clauses and anything else does not.
                return true;
            }
            if Self::word_in(previous, document, CLAUSE_COORDINATORS) {
                return true;
            }
            if !Self::modifies_the_clause(previous, document) {
                return false;
            }
            cursor -= 1;
        }
        // Reaching the start of the sentence: nothing to separate.
        true
    }

    /// May this word stand between the comma and the conjunction?
    ///
    /// Anything that can be read as an adverb or an adjective, and nothing that
    /// can only be a noun or a verb. Those cannot open a clause of their own, so
    /// a comma in front of them is the comma this rule is looking for.
    ///
    /// A word with *both* readings does not count, even though `teils` is an
    /// adverb as well as the genitive of `Teil` and is the adverb in *», teils
    /// weil es Dokumentationen gibt«*. Accepting any adverb reading was tried
    /// and is a bad trade: German separable prefixes are adverbs too, so *»Sie
    /// rief an nachdem sie angekommen war«* walks past `an` to a verb it should
    /// have stopped at. Two corpus false positives fewer, four real missing
    /// commas missed.
    fn modifies_the_clause(token: &Token, document: &Document) -> bool {
        if Self::is_focus_particle(token, document) {
            return true;
        }
        let TokenKind::Word(Some(metadata)) = &token.kind else {
            // An unknown word could be anything, including a noun.
            return false;
        };
        !metadata.is_noun() && !metadata.is_verb() && !metadata.is_proper_noun()
    }

    /// Is the conjunction being *talked about* rather than used?
    ///
    /// German writes that as a hyphenated compound — *»den obwohl-Satz«*, *»die
    /// weil-Konstruktion«* — and a linguistics article is full of them.
    fn is_hyphenated_mention(tokens: &[&Token], index: usize) -> bool {
        tokens
            .get(index + 1)
            .is_some_and(|next| matches!(next.kind, TokenKind::Punctuation(Punctuation::Hyphen)))
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

                // The conjunction as a word, not as a conjunction.
                if Self::is_hyphenated_mention(&tokens, index) {
                    continue;
                }

                // Part of a fixed phrase that is not a clause.
                let conjunction_text: String =
                    document.get_span_content(&token.span).iter().collect();
                if FIXED_PHRASES.iter().any(|(first, second)| {
                    *second == conjunction_text && Self::word_in(previous, document, &[first])
                }) {
                    continue;
                }

                // The comma may belong further left, in front of the particles
                // or the coordinating conjunction that govern the clause.
                if Self::comma_is_further_left(&tokens, index, document) {
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

    /// German puts a modal particle or a focus adverb between the comma and the
    /// conjunction, and the comma is then correct where it stands. Every one of
    /// these came out of a corpus of published prose, where looking exactly one
    /// token to the left reported them as missing commas.
    #[test]
    fn a_particle_may_stand_between_the_comma_and_the_conjunction() {
        for text in [
            "Im Lexikon fehlt der Eintrag, wohl weil man sich sonst verlieren würde.",
            "Er stieg auf den Berg, einfach weil der Berg da ist.",
            "Die Planung wurde verschoben, vermutlich weil die Kosten stiegen.",
            "Etwas anderes gilt nur dann, etwa weil er davon wusste.",
            "Sie führte Gespräche, ganz einfach weil es ihr Spaß machte.",
        ] {
            assert_eq!(lint_count(text), 0, "should not fire on {text:?}");
        }
    }

    /// A noun or a verb in between is a clause of its own, and the comma really
    /// is missing.
    #[test]
    fn a_noun_between_them_does_not_license_the_comma() {
        for text in [
            "Die Nanostruktur ist von Bedeutung weil große Oberflächen entstehen.",
            "Er ging nach Hause und blieb dort weil er krank war.",
        ] {
            assert_eq!(lint_count(text), 1, "should fire on {text:?}");
        }
    }

    /// Talking about the conjunction rather than using it. German writes that as
    /// a hyphenated compound, and a linguistics article is full of them.
    #[test]
    fn a_hyphenated_mention_is_not_a_clause() {
        for text in [
            "Die Ersetzung erweist den obwohl-Satz als Satzglied.",
            "Er untersucht die weil-Konstruktion im Neuhochdeutschen.",
        ] {
            assert_eq!(lint_count(text), 0, "should not fire on {text:?}");
        }
    }
}
