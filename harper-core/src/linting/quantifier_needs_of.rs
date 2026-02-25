use crate::Token;
use crate::expr::{Expr, SequenceExpr};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

/// Flags phrases like `a couple months` → should be `a couple **of** months`.
pub struct QuantifierNeedsOf {
    expr: Box<dyn Expr>,
}

impl Default for QuantifierNeedsOf {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .then_indefinite_article()
            .t_ws()
            .then_word_set(&["couple", "lot"])
            .t_ws()
            .then_plural_nominal();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for QuantifierNeedsOf {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        Some(Lint {
            span: matched_tokens.get(2)?.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::InsertAfter(" of".chars().collect())],
            message: "Add `of` in this quantity phrase.".to_owned(),
            priority: 32,
        })
    }

    fn description(&self) -> &'static str {
        "Detects missing `of` after the quantifier “a couple” when it precedes a plural noun"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::QuantifierNeedsOf;

    #[test]
    fn fixes_a_couple_months() {
        assert_suggestion_result(
            "A couple months ago...",
            QuantifierNeedsOf::default(),
            "A couple of months ago...",
        );
    }

    #[test]
    fn fixes_a_couple_weeks() {
        assert_suggestion_result(
            "A couple weeks ago...",
            QuantifierNeedsOf::default(),
            "A couple of weeks ago...",
        );
    }

    #[test]
    fn fixes_a_couple_days() {
        assert_suggestion_result(
            "A couple days ago...",
            QuantifierNeedsOf::default(),
            "A couple of days ago...",
        );
    }

    #[test]
    fn fixes_a_couple_seconds() {
        assert_suggestion_result(
            "A couple seconds ago...",
            QuantifierNeedsOf::default(),
            "A couple of seconds ago...",
        );
    }

    #[test]
    fn fixes_a_couple_minutes() {
        assert_suggestion_result(
            "A couple minutes ago...",
            QuantifierNeedsOf::default(),
            "A couple of minutes ago...",
        );
    }

    #[test]
    fn fixes_a_couple_houses() {
        assert_suggestion_result(
            "A couple houses ago...",
            QuantifierNeedsOf::default(),
            "A couple of houses ago...",
        );
    }

    #[test]
    fn fixes_a_couple_centuries() {
        assert_suggestion_result(
            "A couple centuries ago...",
            QuantifierNeedsOf::default(),
            "A couple of centuries ago...",
        );
    }

    #[test]
    fn fixes_a_couple_people() {
        assert_suggestion_result(
            "I saw a couple people walk by a few minutes ago.",
            QuantifierNeedsOf::default(),
            "I saw a couple of people walk by a few minutes ago.",
        );
    }
}
