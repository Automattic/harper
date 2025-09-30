use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind};

pub struct QuoteSpacing {
    expr: Box<dyn Expr>,
}

impl Default for QuoteSpacing {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .then_any_word()
            .then_quote()
            .then_any_word();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for QuoteSpacing {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        dbg!(matched_tokens);
        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Formatting,
            suggestions: vec![],
            message: "A quote must be preceded or succeeded by a space.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Checks that quotation marks are preceded or succeeded by whitespace."
    }
}

#[cfg(test)]
mod tests {
    use super::QuoteSpacing;
    use crate::linting::tests::{assert_lint_count, assert_no_lints};

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
}
