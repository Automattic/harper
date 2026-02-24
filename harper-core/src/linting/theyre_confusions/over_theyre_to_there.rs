use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    expr::SequenceExpr,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};
use super::token_is_theyre;

pub struct OverTheyreToThere {
    expr: Box<dyn crate::expr::Expr>,
}

impl Default for OverTheyreToThere {
    fn default() -> Self {
        let expr = SequenceExpr::aco("over")
            .t_ws()
            .then(token_is_theyre as fn(&Token, &[char]) -> bool);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for OverTheyreToThere {
    type Unit = Chunk;

    fn expr(&self) -> &dyn crate::expr::Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offender = matched_tokens.last()?;
        let template = offender.span.get_content(source);

        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case_str("there", template)],
            message: "Did you mean `there`?".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects locative `over they're` to `over there`."
    }
}

#[cfg(test)]
mod tests {
    use super::OverTheyreToThere;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_locative_ascii_apostrophe() {
        assert_suggestion_result(
            "Put the chairs over they're by the window.",
            OverTheyreToThere::default(),
            "Put the chairs over there by the window.",
        );
    }

    #[test]
    fn corrects_locative_smart_apostrophe() {
        assert_suggestion_result(
            "Is that their car parked over they’re?",
            OverTheyreToThere::default(),
            "Is that their car parked over there?",
        );
    }

    #[test]
    fn ignores_correct_form() {
        assert_lint_count(
            "Put the chairs over there by the window.",
            OverTheyreToThere::default(),
            0,
        );
    }
}
