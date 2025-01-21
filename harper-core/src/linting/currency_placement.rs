use itertools::Itertools;

use crate::{remove_overlaps, Document, Token, TokenStringExt};

use super::{Lint, LintKind, Linter, Suggestion};

#[derive(Debug, Default)]
pub struct CurrencyPlacement {}

impl Linter for CurrencyPlacement {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            for (a, b) in chunk.iter().tuple_windows() {
                lints.extend(generate_lint_for_tokens(*a, *b, document));
            }

            for (p, a, b, c) in chunk.iter().tuple_windows() {
                if !b.kind.is_whitespace() || p.kind.is_currency() {
                    continue;
                }

                lints.extend(generate_lint_for_tokens(*a, *c, document));
            }
        }

        remove_overlaps(&mut lints);

        lints
    }

    fn description(&self) -> &str {
        "The location of currency symbols varies by country. The rule looks for and corrects improper positioning."
    }
}

// Given two tokens that may have an error, check if they do and create a [`Lint`].
fn generate_lint_for_tokens(a: Token, b: Token, document: &Document) -> Option<Lint> {
    let matched_tokens = [a, b];

    let punct = matched_tokens
        .first_punctuation()?
        .kind
        .expect_punctuation();
    let currency = punct.as_currency()?;

    let (value, suffix) = matched_tokens.first_number()?.kind.expect_number();

    let span = matched_tokens.span().unwrap();

    let correct: Vec<_> = currency
        .format_amount(value.into(), suffix)
        .chars()
        .collect();
    let actual = document.get_span_content(span);

    if correct != actual {
        Some(Lint {
            span,
            lint_kind: LintKind::Formatting,
            suggestions: vec![Suggestion::ReplaceWith(correct)],
            message: "The position of the currency symbol matters.".to_string(),
            priority: 63,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::CurrencyPlacement;

    #[test]
    fn eof() {
        assert_suggestion_result(
            "It was my last bill worth more than 4$.",
            CurrencyPlacement::default(),
            "It was my last bill worth more than $4.",
        );
    }

    #[test]
    fn blog_title_allows_correct() {
        assert_lint_count("The Best $25 I Ever Spent", CurrencyPlacement::default(), 0);
    }

    #[test]
    fn blog_title() {
        assert_suggestion_result(
            "The Best 25$ I Ever Spent",
            CurrencyPlacement::default(),
            "The Best $25 I Ever Spent",
        );
    }

    #[test]
    fn blog_title_cents() {
        assert_suggestion_result(
            "The Best ¢25 I Ever Spent",
            CurrencyPlacement::default(),
            "The Best 25¢ I Ever Spent",
        );
    }

    #[test]
    fn blog_title_with_space() {
        assert_suggestion_result(
            "The Best 25   $ I Ever Spent",
            CurrencyPlacement::default(),
            "The Best $25 I Ever Spent",
        );
    }

    #[test]
    fn multiple_dollar() {
        assert_suggestion_result(
            "They were either 25$ 24$ or 23$.",
            CurrencyPlacement::default(),
            "They were either $25 $24 or $23.",
        );
    }

    #[test]
    fn multiple_pound() {
        assert_suggestion_result(
            "They were either 25£ 24£ or 23£.",
            CurrencyPlacement::default(),
            "They were either £25 £24 or £23.",
        );
    }

    #[test]
    fn suffix() {
        assert_suggestion_result(
            "It was my 20th$.",
            CurrencyPlacement::default(),
            "It was my $20th.",
        );
    }
}
