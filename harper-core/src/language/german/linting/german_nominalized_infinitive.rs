use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Token, TokenKind, TokenStringExt, document::Document};

/// Prepositions fused with the article `dem`.
///
/// The article is what makes the following infinitive a noun: *beim Laufen*
/// is *bei dem Laufen*. `am` is left out because it also forms the superlative
/// (*am schnellsten*), which a verb reading on the next word does not rule
/// out.
const FUSED_PREPOSITIONS: &[&str] = &["beim", "zum", "vom"];

/// Infinitive-shaped words that stay lower case after a fused preposition:
/// *zum einen …, zum anderen …*.
const FIXED_LOWERCASE: &[&str] = &["einen", "anderen"];

/// Sentence-initial determiners that make a following infinitive the subject:
/// *Das essen war lecker*, *Mein lesen ist langsam*.
const OPENING_DETERMINERS: &[&str] = &[
    "das", "mein", "dein", "sein", "ihr", "unser", "euer", "dieses", "jedes",
];

/// Finite verbs in the third person singular. With one of these later in the
/// same clause, the infinitive after a sentence-initial *Das* cannot itself be
/// the finite verb — *Das essen wir* has no second one — so it is the subject.
const SINGULAR_FINITE: &[&str] = &[
    "ist",
    "war",
    "wäre",
    "sei",
    "hat",
    "hatte",
    "hätte",
    "wird",
    "wurde",
    "würde",
    "kann",
    "konnte",
    "könnte",
    "muss",
    "musste",
    "müsste",
    "soll",
    "sollte",
    "darf",
    "durfte",
    "mag",
    "macht",
    "machte",
    "tut",
    "tat",
    "kostet",
    "kostete",
    "dauert",
    "dauerte",
    "gefällt",
    "gefiel",
    "fällt",
    "fiel",
    "schmeckt",
    "schmeckte",
    "bleibt",
    "blieb",
    "scheint",
    "schien",
    "gilt",
    "galt",
    "bringt",
    "brachte",
    "klappt",
    "klappte",
    "hilft",
    "half",
    "beginnt",
    "begann",
    "endet",
    "endete",
];

/// Capitalizes the infinitive after *beim*, *zum* and *vom*: *beim laufen* ->
/// *beim Laufen*, *zum essen* -> *zum Essen*.
///
/// The fused article turns the infinitive into a noun, so the capital is
/// obligatory, and it is one of the most frequent mistakes in school writing.
/// `GermanNounCapitalization` cannot see it: every `-en` word is a verb form
/// first, and that rule rejects the shape outright.
///
/// Three things keep correct text out:
///
/// * the word must have a verb reading and no adjective reading. *beim
///   nächsten Mal* and *zum ersten Mal* are adjectives.
/// * a capitalized word right after it makes it attributive: *beim schnellen
///   Laufen* is an adjective in front of the noun.
/// * the closed list above, and a minimum length: `oben` carries a stray verb
///   reading, and no four-letter infinitive in `-en` is worth the risk.
#[derive(Default)]
pub struct GermanNominalizedInfinitive;

impl GermanNominalizedInfinitive {
    fn lowercase(token: &Token, document: &Document) -> String {
        document
            .get_span_content(&token.span)
            .iter()
            .flat_map(|c| c.to_lowercase())
            .collect()
    }

    fn is_candidate(tokens: &[&Token], index: usize, document: &Document) -> bool {
        let token = tokens[index];
        let TokenKind::Word(Some(meta)) = &token.kind else {
            return false;
        };

        let chars = document.get_span_content(&token.span);
        let word: String = chars.iter().collect();
        if !chars.first().is_some_and(|c| c.is_lowercase())
            || !chars.iter().all(|c| c.is_alphabetic())
            || word.chars().count() < 5
            || !["en", "ern", "eln"].iter().any(|e| word.ends_with(e))
            || FIXED_LOWERCASE.contains(&word.as_str())
        {
            return false;
        }

        // *vom oben erwähnten*, *vom gegen ihn gerichteten*: adverbs and
        // prepositions with a stray verb reading open an extended attribute.
        if !meta.is_verb() || meta.is_adjective() || meta.is_adverb() || meta.preposition {
            return false;
        }

        // *beim bekommen-Passiv*: the first half of a hyphenated compound.
        if tokens
            .get(index + 1)
            .is_some_and(|next| next.kind.is_hyphen() && next.span.start == token.span.end)
        {
            return false;
        }

        let Some(prev) = index.checked_sub(1).map(|i| tokens[i]) else {
            return false;
        };
        let preposition = Self::lowercase(prev, document);
        // *Ich bin am lesen*: the progressive with *am* ends its clause, which
        // the superlative (*am schnellsten*, an adjective anyway) does not
        // need to, so *am* is read only there.
        let progressive = preposition == "am"
            && !word.ends_with("sten")
            && tokens
                .get(index + 1)
                .is_none_or(|next| matches!(next.kind, TokenKind::Punctuation(_)));
        if !FUSED_PREPOSITIONS.contains(&preposition.as_str()) && !progressive {
            return false;
        }

        let next_is_capitalized = tokens.get(index + 1).is_some_and(|next| {
            next.kind.is_word()
                && document
                    .get_span_content(&next.span)
                    .first()
                    .is_some_and(|c| c.is_uppercase())
        });
        !next_is_capitalized
    }

    /// *Das essen in der Kantine war lecker*: an infinitive right behind a
    /// sentence-initial determiner, with a singular finite verb later in the
    /// same clause.
    fn is_subject_infinitive(tokens: &[&Token], document: &Document) -> bool {
        let Some(TokenKind::Word(Some(meta))) = tokens.get(1).map(|t| &t.kind) else {
            return false;
        };
        let chars = document.get_span_content(&tokens[1].span);
        let word: String = chars.iter().collect();
        if !OPENING_DETERMINERS.contains(&Self::lowercase(tokens[0], document).as_str())
            || !chars.first().is_some_and(|c| c.is_lowercase())
            || !chars.iter().all(|c| c.is_alphabetic())
            || word.chars().count() < 5
            || !["en", "ern", "eln"].iter().any(|e| word.ends_with(e))
            || !meta.is_verb()
            || meta.is_adjective()
            || meta.is_adverb()
            || meta.preposition
        {
            return false;
        }
        tokens[2..]
            .iter()
            .take_while(|token| !matches!(token.kind, TokenKind::Punctuation(_)))
            .map(|token| Self::lowercase(token, document))
            .take_while(|word| !matches!(word.as_str(), "und" | "oder" | "aber" | "denn"))
            .any(|word| SINGULAR_FINITE.contains(&word.as_str()))
    }
}

impl Linter for GermanNominalizedInfinitive {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let tokens: Vec<&Token> = sentence
                .iter()
                .filter(|t| !t.kind.is_whitespace())
                .collect();

            for index in 0..tokens.len() {
                if !Self::is_candidate(&tokens, index, document)
                    && !(index == 1 && Self::is_subject_infinitive(&tokens, document))
                {
                    continue;
                }

                let token = tokens[index];
                let word: String = document.get_span_content(&token.span).iter().collect();
                let mut chars = word.chars();
                let fixed: String = chars
                    .next()
                    .into_iter()
                    .flat_map(|c| c.to_uppercase())
                    .chain(chars)
                    .collect();
                let prev: String = document
                    .get_span_content(&tokens[index - 1].span)
                    .iter()
                    .collect();

                lints.push(Lint {
                    span: token.span,
                    lint_kind: LintKind::Capitalization,
                    suggestions: vec![Suggestion::ReplaceWith(fixed.chars().collect())],
                    message: format!(
                        "Nach »{prev}« ist »{word}« ein Nomen und wird großgeschrieben: »{fixed}«."
                    ),
                    priority: 25,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Schreibt den Infinitiv nach »beim«, »zum«, »vom« und als Subjekt nach »Das« groß (»beim Laufen«, »Das Essen war gut«)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanNominalizedInfinitive;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn lint_count(text: &str) -> usize {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanNominalizedInfinitive.lint(&document).len()
    }

    #[test]
    fn capitalizes_the_infinitive() {
        for text in [
            "Beim laufen tun mir die Füße weh.",
            "Die Schuhe sind zum wandern gedacht.",
            "Er ist vom rennen müde.",
            "Ich brauche Schuhe zum laufen.",
            "Beim schnüren der Schuhe hilft er mir.",
            "Das Leder ist zum reparieren zu alt.",
            "Wir trafen uns beim einkaufen.",
            "Ich bin gerade am lesen.",
            "Wir waren am essen, als er kam.",
            "Das essen in der Kantine war lecker.",
            "Das schwimmen im See ist verboten.",
            "Mein zeichnen wird immer besser.",
        ] {
            assert_eq!(lint_count(text), 1, "should fire on {text:?}");
        }
    }

    #[test]
    fn leaves_correct_text_alone() {
        for text in [
            "Er läuft am schnellsten.",
            "Das gefällt mir am besten.",
            "Am laufenden Band.",
            "Beim Laufen tun mir die Füße weh.",
            "Die Schuhe sind zum Wandern gedacht.",
            "Beim schnellen Laufen schwitzt man.",
            "Zum ersten Mal trage ich Stiefel.",
            "Beim nächsten Mal kaufe ich Sandalen.",
            "Zum einen sind sie billig, zum anderen bequem.",
            "Er ging zum Schuhmacher.",
            "Wir laufen zum Bahnhof.",
            "Der Plan wurde vom oben dargestellten Plan abgeleitet.",
            "Dabei kommt es zum oben erwähnten Wandel.",
            "Sie erzählte vom gegen ihn gerichteten Verdacht.",
            "Das gilt beim bekommen-Passiv.",
            "Das essen wir morgen.",
            "Das wissen alle, die da war.",
            "Das essen wir, wenn es fertig ist.",
            "Das Essen in der Kantine war lecker.",
            "Das wollen wir und das ist gut.",
            "Das neue Haus ist schön.",
        ] {
            assert_eq!(lint_count(text), 0, "should not fire on {text:?}");
        }
    }
}
