/*
let message "Did you mean `by accident`?"
let description "Incorrect preposition: `by accident` is the idiomatic expression."
 */

use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct ByAccident {
    expr: Box<dyn Expr>,
}

impl Default for ByAccident {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::aco("on")
                    .t_ws()
                    .then_optional(
                        SequenceExpr::word_set(&[
                            "complete", "happy", "literal", "mere", "pure", "sheer", "total",
                        ])
                        .t_ws(),
                    )
                    .t_aco("accident"),
            ),
        }
    }
}

impl ExprLinter for ByAccident {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Incorrect preposition: `by accident` is the idiomatic expression."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.first()?.span;
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "by",
            span.get_content(src),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "Did you mean `by accident`?".to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ByAccident;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn fix_on_accident() {
        assert_suggestion_result(
            "Snapshot revert feature is unintuitive and easy to use on accident.",
            ByAccident::default(),
            "Snapshot revert feature is unintuitive and easy to use by accident.",
        );
    }

    #[test]
    fn fix_on_complete_accident() {
        assert_suggestion_result(
            "I Came across this comment on complete accident, however, I did notice the same thing with a slowdown in chrome for android",
            ByAccident::default(),
            "I Came across this comment by complete accident, however, I did notice the same thing with a slowdown in chrome for android",
        );
    }

    #[test]
    fn fix_on_happy_accident() {
        assert_suggestion_result(
            "Just did this on happy accident the other day with my partner.",
            ByAccident::default(),
            "Just did this by happy accident the other day with my partner.",
        );
    }

    #[test]
    fn fix_on_literal_accident() {
        assert_suggestion_result(
            "I did this on literal accident, trying to prove someone wrong that its not that easy.",
            ByAccident::default(),
            "I did this by literal accident, trying to prove someone wrong that its not that easy.",
        );
    }

    #[test]
    fn fix_on_mere_accident() {
        assert_suggestion_result(
            "I hated this challenge and nope I don't I completed it on mere accident.",
            ByAccident::default(),
            "I hated this challenge and nope I don't I completed it by mere accident.",
        );
    }

    #[test]
    fn fix_on_pure_accident() {
        assert_suggestion_result(
            "I got this on pure accident after forgetting to enable WebGL on LibreWolf",
            ByAccident::default(),
            "I got this by pure accident after forgetting to enable WebGL on LibreWolf",
        );
    }

    #[test]
    fn fix_on_sheer_accident() {
        assert_suggestion_result(
            "I more of think of things that got discovered on sheer accident, something no normal human would just do and expect results.",
            ByAccident::default(),
            "I more of think of things that got discovered by sheer accident, something no normal human would just do and expect results.",
        );
    }

    #[test]
    fn fix_on_total_accident() {
        assert_suggestion_result(
            "On Total Accident, I Found Out Yona's True Title.",
            ByAccident::default(),
            "By Total Accident, I Found Out Yona's True Title.",
        );
    }
}
