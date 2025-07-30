use crate::Token;
use crate::expr::Expr;
use crate::expr::SequenceExpr;

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct SomewhatSomething {
    expr: Box<dyn Expr>,
}

impl Default for SomewhatSomething {
    fn default() -> Self {
        let pattern = SequenceExpr::aco("somewhat")
            .then_whitespace()
            .t_aco("of")
            .then_whitespace()
            .t_aco("a");

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for SomewhatSomething {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;
        let og = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::Style,
            suggestions: vec![Suggestion::replace_with_match_case_str("something", og)],
            message: "Use the traditional form.".to_owned(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "When describing a single instance of a noun, use `something` rather than `somewhat`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::SomewhatSomething;

    #[test]
    fn issue_414() {
        assert_suggestion_result(
            "This may be somewhat of a surprise.",
            SomewhatSomething::default(),
            "This may be something of a surprise.",
        );
    }
}
