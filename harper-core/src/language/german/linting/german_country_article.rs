//! *aus Türkei*, *in Schweiz*: the countries German names with an article.

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::grammar::noun_phrase::lowercase_of,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Feminine country and region names, which always take the article: *die
/// Türkei*, *in der Schweiz*, *aus der Ukraine*. Most country names are neuter
/// and take none (*aus Polen*), which is why learners drop it here too.
const FEMININE: &[&str] = &[
    "Türkei",
    "Schweiz",
    "Slowakei",
    "Mongolei",
    "Ukraine",
    "Tschechei",
    "Walachei",
    "Arktis",
    "Antarktis",
    "Karibik",
    "Pfalz",
    "Lausitz",
    "Toskana",
    "Normandie",
    "Bretagne",
    "Provence",
    "Krim",
    "Sahara",
];

/// Plural names: *die USA*, *in den Niederlanden*.
const PLURAL: &[&str] = &[
    "USA",
    "Niederlande",
    "Niederlanden",
    "Philippinen",
    "Malediven",
    "Seychellen",
    "Bahamas",
    "Azoren",
    "Kanaren",
    "Komoren",
];

/// What each preposition makes of the article, for a feminine and a plural
/// name. *in* and *nach* both ask "where to?" in some sentences and "where?"
/// in others; *nach* is always direction (*in die Türkei*), *in* is offered
/// both ways.
const ARTICLES: &[(&str, &[&str], &[&str])] = &[
    ("aus", &["aus der"], &["aus den"]),
    ("in", &["in der", "in die"], &["in den", "in die"]),
    ("nach", &["in die"], &["in die"]),
    ("von", &["von der"], &["von den"]),
    ("mit", &["mit der"], &["mit den"]),
    ("bei", &["bei der"], &["bei den"]),
    ("zu", &["zur"], &["zu den"]),
    ("für", &["für die"], &["für die"]),
    ("durch", &["durch die"], &["durch die"]),
    ("über", &["über die"], &["über die"]),
    ("gegen", &["gegen die"], &["gegen die"]),
];

/// Requires the article in front of a country name that has one: *aus
/// Türkei* → *aus der Türkei*, *nach Schweiz* → *in die Schweiz*, *in USA* →
/// *in den USA*.
///
/// Only directly behind a preposition, where the article is certain to be
/// missing. *Türkei-Reise* and *Schweiz-Besuch* are compounds (a hyphen
/// follows) and are left alone.
#[derive(Default)]
pub struct GermanCountryArticle;

impl Linter for GermanCountryArticle {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let content = document.get_full_content();

        for sentence in document.iter_sentences() {
            let words: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();

            for pair in words.windows(2) {
                let [preposition, name] = pair else { continue };
                if !matches!(preposition.kind, TokenKind::Word(_))
                    || !matches!(name.kind, TokenKind::Word(_))
                {
                    continue;
                }
                let name_text: String = document.get_span_content(&name.span).iter().collect();
                let plural = PLURAL.contains(&name_text.as_str());
                if !plural && !FEMININE.contains(&name_text.as_str()) {
                    continue;
                }
                if content.get(name.span.end) == Some(&'-') {
                    continue;
                }
                let preposition_text = lowercase_of(preposition, document);
                let Some((_, feminine, plural_forms)) = ARTICLES
                    .iter()
                    .find(|(word, _, _)| *word == preposition_text)
                else {
                    continue;
                };
                let forms = if plural { plural_forms } else { feminine };

                let span = preposition.span;
                let original = document.get_span_content(&span);
                lints.push(Lint {
                    span,
                    lint_kind: LintKind::Grammar,
                    suggestions: forms
                        .iter()
                        .map(|form| {
                            Suggestion::replace_with_match_case(form.chars().collect(), original)
                        })
                        .collect(),
                    priority: 31,
                    message: format!("»{name_text}« steht mit Artikel: »die {name_text}«."),
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Ergänzt den Artikel vor Ländernamen, die einen haben (»in der Schweiz«, »aus der Türkei«)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanCountryArticle;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn fixes(text: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanCountryArticle
            .lint(&document)
            .into_iter()
            .flat_map(|lint| lint.suggestions)
            .map(|suggestion| suggestion.to_string())
            .collect()
    }

    #[test]
    fn a_missing_article_is_reported() {
        assert_eq!(fixes("Er kommt aus Türkei."), ["Replace with: “aus der”"]);
        assert_eq!(
            fixes("Wir fahren nach Schweiz."),
            ["Replace with: “in die”"]
        );
        assert_eq!(
            fixes("Sie wohnt in USA."),
            ["Replace with: “in den”", "Replace with: “in die”"]
        );
        assert_eq!(fixes("Das kommt von Ukraine."), ["Replace with: “von der”"]);
    }

    #[test]
    fn the_article_already_there_is_quiet() {
        for text in [
            "Er kommt aus der Türkei.",
            "Wir fahren in die Schweiz.",
            "Sie wohnt in den USA.",
            "Wir fahren nach Polen.",
            "Er plant eine Türkei-Reise.",
            "Die Schweiz ist schön.",
        ] {
            assert!(fixes(text).is_empty(), "{text}");
        }
    }
}
