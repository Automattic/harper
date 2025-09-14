use crate::Token;
use crate::expr::{Expr, SequenceExpr};

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
            message: "Consider using `something of a` in more formal writing.".to_owned(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Flags the phrase `somewhat of a` in favor of `something of a`, which can be considered more traditional."
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

    #[test]
    fn flag_these() {
        assert_suggestion_result(
            "These are somewhat of a cult data structure.",
            SomewhatSomething::default(),
            "These are something of a cult data structure.",
        );
    }
}
