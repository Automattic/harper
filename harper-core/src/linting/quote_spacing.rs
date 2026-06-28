use crate::expr::{ExprExt, SequenceExpr};
use crate::linting::LintKind;
use crate::{Document, TokenStringExt};

use super::{Lint, Linter};

pub struct QuoteSpacing {
    expr: SequenceExpr,
}

impl QuoteSpacing {
    pub fn new() -> Self {
        Self {
            expr: SequenceExpr::any_word().then_quote().then_any_word(),
        }
    }
}

impl Default for QuoteSpacing {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for QuoteSpacing {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for (index, m) in self.expr.iter_matches_in_doc(document).enumerate() {
            let matched_tokens = m.get_content(document.get_tokens());

            let Some(span) = matched_tokens.span() else {
                continue;
            };

            let mut suggestions = vec![
                super::Suggestion::ReplaceWith(vec![' ', '"']),
                super::Suggestion::ReplaceWith(vec!['"', ' ']),
            ];

            // Every odd-numbered quote is likely a close quote, ie space likely should be after it.
            if index % 2 == 1 {
                suggestions.swap(0, 1);
            }

            lints.push(Lint {
                span: matched_tokens[1].span,
                lint_kind: LintKind::Formatting,
                suggestions,
                message: "A quote must be preceded or succeeded by a space.".to_owned(),
                priority: 31,
            })
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks that quotation marks are preceded or succeeded by whitespace."
    }
}

#[cfg(test)]
mod tests {
    use super::QuoteSpacing;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn flags_missing_space_before_quote() {
        assert_lint_count("He said\"hello\" to me.", QuoteSpacing::default(), 1);
    }

    #[test]
    fn flags_missing_space_after_quote() {
        assert_lint_count(
            "She whispered \"hurry\"and left.",
            QuoteSpacing::default(),
            1,
        );
    }

    #[test]
    fn allows_quotes_with_spacing() {
        assert_no_lints("They shouted \"charge\" together.", QuoteSpacing::default());
    }

    #[test]
    fn allows_quotes_at_end_of_sentence() {
        assert_no_lints("They shouted \"charge.\"", QuoteSpacing::default());
    }

    #[test]
    fn fix_by_inserting_space_after() {
        assert_suggestion_result(
            "It's not a complete sentence since it lacks a verb but it would be valid as a title or answer to a question since \"dog runs\"are things which exist",
            QuoteSpacing::default(),
            "It's not a complete sentence since it lacks a verb but it would be valid as a title or answer to a question since \"dog runs\" are things which exist",
        );
    }
}
