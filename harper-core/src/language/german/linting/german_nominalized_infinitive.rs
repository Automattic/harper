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
        if !FUSED_PREPOSITIONS.contains(&Self::lowercase(prev, document).as_str()) {
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
                if !Self::is_candidate(&tokens, index, document) {
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
        "Schreibt den Infinitiv nach »beim«, »zum« und »vom« groß (»beim Laufen«)."
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
        ] {
            assert_eq!(lint_count(text), 1, "should fire on {text:?}");
        }
    }

    #[test]
    fn leaves_correct_text_alone() {
        for text in [
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
        ] {
            assert_eq!(lint_count(text), 0, "should not fire on {text:?}");
        }
    }
}
