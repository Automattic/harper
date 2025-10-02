//! Linter for correcting common errors with the phrase "in [adjective] condition".

use crate::{
    Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    token_string_ext::TokenStringExt,
};

/// Linter that corrects common errors with phrases like "in good condition".
///
/// Handles two cases:
/// 1. "in [a/an] [adjective] condition" -> "in [adjective] condition"
/// 2. "in [adjective] conditions" -> "in [adjective] condition"
pub struct InAdjectiveCondition {
    expr: Box<dyn Expr>,
}

impl Default for InAdjectiveCondition {
    fn default() -> Self {
        let singular = SequenceExpr::default()
            .then_indefinite_article()
            .t_ws()
            .then_adjective()
            .t_ws()
            .t_aco("condition");

        let plural = SequenceExpr::default()
            .then_adjective()
            .t_ws()
            .t_aco("conditions");

        Self {
            expr: Box::new(
                SequenceExpr::default()
                    .t_aco("in")
                    .t_ws()
                    .then_any_of(vec![Box::new(singular), Box::new(plural)]),
            ),
        }
    }
}

impl ExprLinter for InAdjectiveCondition {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (span, sugg, msg) = match toks.len() {
            5 => (
                toks.last()?.span,
                Suggestion::replace_with_match_case(
                    "condition".chars().collect(),
                    toks.last()?.span.get_content(src),
                ),
                "`Condition` should be singular.",
            ),
            7 => (
                toks[1..3].span()?,
                Suggestion::Remove,
                "An indefinite article should not be used here.",
            ),
            _ => return None,
        };

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![sugg],
            message: msg.to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects incorrect variants of `in good condition` with an indefinite article or plural."
    }
}

#[cfg(test)]
mod tests {
    use super::InAdjectiveCondition;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn fix_in_good_conditions() {
        assert_suggestion_result(
            "in good conditions",
            InAdjectiveCondition::default(),
            "in good condition",
        );
    }

    #[test]
    fn fix_in_a_bad_condition() {
        assert_suggestion_result(
            "in a bad condition",
            InAdjectiveCondition::default(),
            "in bad condition",
        );
    }

    #[test]
    fn fix_great_condition_all_caps() {
        assert_suggestion_result(
            "YEAH IT'S IN GREAT CONDITIONS REALLY!",
            InAdjectiveCondition::default(),
            "YEAH IT'S IN GREAT CONDITION REALLY!",
        )
    }
}
