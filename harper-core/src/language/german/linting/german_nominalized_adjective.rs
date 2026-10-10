//! *etwas Gutes*, *nichts Neues*, *alles Gute*: an adjective that a quantifier
//! turns into a noun is written with a capital.

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::grammar::noun_phrase::lowercase_of,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// The indefinite quantifiers after which an adjective stands as a noun, with
/// the ending the adjective takes behind each: *etwas Gutes*, *nichts Neues*,
/// *viel Kluges*, *wenig Erfreuliches*, *allerlei Nützliches*, *genug
/// Schlechtes* — and *alles Gute*, where the article-like *alles* takes the
/// weak ending. Duden, Amtliche Regelung § 57 (1).
const QUANTIFIERS: &[(&str, &str)] = &[
    ("etwas", "es"),
    ("nichts", "es"),
    ("viel", "es"),
    ("wenig", "es"),
    ("allerlei", "es"),
    ("genug", "es"),
    ("alles", "e"),
];

/// The adjectives that stay lower case all the same, because they work as
/// pronouns: *etwas anderes*, *alles andere*, *nichts weniger*. § 58 (5)
/// lists *ander-*, *ein-*, *viel-* and *wenig-*; matched as stems, so the
/// declined forms are covered, and the contracted *andres* with them.
const PRONOMINAL_STEMS: &[&str] = &["ander", "andr", "ein", "viel", "wenig"];

/// Reports the lower-case adjective behind *etwas*, *nichts*, *viel*,
/// *wenig*, *allerlei*, *genug* and *alles*: *nichts neues* → *nichts Neues*,
/// *alles gute* → *alles Gute*.
///
/// The adjective is only a noun when no noun follows it. *etwas seltsames
/// Gefühl* is an attribute — and *etwas* the adverb "somewhat" — so a
/// capitalized word behind it, or another adjective joined by a comma or
/// *und*, leaves it alone.
#[derive(Default)]
pub struct GermanNominalizedAdjective;

impl GermanNominalizedAdjective {
    fn is_lowercase(token: &Token, document: &Document) -> bool {
        document
            .get_span_content(&token.span)
            .first()
            .is_some_and(|c| c.is_lowercase())
    }

    /// A lower-case adjective with the ending the quantifier asks for.
    fn is_bare_adjective(token: &Token, ending: &str, document: &Document) -> bool {
        let word = lowercase_of(token, document);
        Self::is_lowercase(token, document)
            && word.ends_with(ending)
            && word.chars().count() > ending.len() + 2
            && !PRONOMINAL_STEMS.iter().any(|stem| word.starts_with(stem))
            && token.kind.is_adjective()
            && !token.kind.is_determiner()
            && !token.kind.is_pronoun()
            && !token.kind.is_verb()
            // *alles zwangsweise eine Sache*, *nichts weiter*: an adverb.
            && !token.kind.is_adverb()
    }

    /// Does something behind the adjective make it an attribute? A noun
    /// (*etwas seltsames Gefühl*), another adjective (*etwas schönes neues
    /// Haus*), or one joined to it (*etwas schönes und großes Haus*,
    /// *etwas schönes, großes Haus*).
    fn is_attribute(words: &[&Token], after: usize, document: &Document) -> bool {
        let Some(next) = words.get(after) else {
            return false;
        };
        let joined = matches!(next.kind, TokenKind::Punctuation(_))
            && document.get_span_content(&next.span) == [',']
            || matches!(lowercase_of(next, document).as_str(), "und" | "oder");
        let next = if joined {
            match words.get(after + 1) {
                Some(next) => next,
                None => return false,
            }
        } else {
            next
        };
        next.kind.is_word()
            && (!Self::is_lowercase(next, document)
                || (next.kind.is_adjective() && !next.kind.is_verb()))
    }
}

impl Linter for GermanNominalizedAdjective {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let words: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();
            for at in 1..words.len() {
                let quantifier = lowercase_of(words[at - 1], document);
                let Some(&(_, ending)) = QUANTIFIERS.iter().find(|(q, _)| *q == quantifier) else {
                    continue;
                };
                let adjective = words[at];
                if !Self::is_bare_adjective(adjective, ending, document)
                    || Self::is_attribute(&words, at + 1, document)
                {
                    continue;
                }
                let original = document.get_span_content(&adjective.span);
                let mut fixed: Vec<char> = original.to_vec();
                if let Some(first) = fixed.first_mut() {
                    *first = first.to_uppercase().next().unwrap_or(*first);
                }
                let fixed_text: String = fixed.iter().collect();
                lints.push(Lint {
                    span: adjective.span,
                    lint_kind: LintKind::Capitalization,
                    suggestions: vec![Suggestion::ReplaceWith(fixed)],
                    message: format!(
                        "Nach »{quantifier}« ist das Adjektiv ein Nomen und wird großgeschrieben: »{quantifier} {fixed_text}«."
                    ),
                    priority: 31,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Schreibt Adjektive nach »etwas«, »nichts«, »viel«, »wenig« und »alles« groß, wenn sie als Nomen stehen: »etwas Gutes«, »alles Gute«."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanNominalizedAdjective;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn flagged(text: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanNominalizedAdjective
            .lint(&document)
            .into_iter()
            .map(|lint| document.get_span_content_str(&lint.span))
            .collect()
    }

    #[test]
    fn an_adjective_after_a_quantifier_is_capitalized() {
        for (text, word) in [
            ("Es gibt nichts neues.", "neues"),
            ("Ich habe etwas besseres zu tun.", "besseres"),
            ("Dahinter verbirgt sich nichts gutes.", "gutes"),
            ("Suchst du etwas bestimmtes?", "bestimmtes"),
            ("Bei dem Treffen wurde viel kluges gesagt.", "kluges"),
            (
                "Ich brauche noch etwas passendes zum Anziehen.",
                "passendes",
            ),
            ("Ich wünsche dir alles gute zum Geburtstag.", "gute"),
            ("Nichts großes geschah.", "großes"),
            ("Es war wenig erfreuliches dabei.", "erfreuliches"),
        ] {
            assert_eq!(flagged(text), [word], "{text}");
        }
    }

    #[test]
    fn an_attribute_or_a_pronoun_is_quiet() {
        for text in [
            "Es gibt nichts Neues.",
            "Ich wünsche dir alles Gute.",
            "Ich habe ein etwas seltsames Gefühl.",
            "Ich kaufe etwas schmackhaftes Brot.",
            "Das ist etwas schönes, großes Haus.",
            "Er hat etwas neues und teures Auto gekauft.",
            "Dahinter verbirgt sich nichts anderes als eine Lüge.",
            "Alles andere ist egal.",
            "Er hat viel gutes Essen mitgebracht.",
            "Es gibt etwas mehr Zeit.",
            "Das ist etwas schwierig.",
            "Alles gehört dir.",
            "Er hat nichts gesagt.",
            "Das ist alles zwangsweise eine Sache der Erziehung.",
        ] {
            assert!(flagged(text).is_empty(), "{text}");
        }
    }
}
