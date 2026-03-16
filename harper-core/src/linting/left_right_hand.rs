use crate::Token;
use crate::expr::Expr;
use crate::expr::SequenceExpr;

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct LeftRightHand {
    expr: Box<dyn Expr>,
}

impl Default for LeftRightHand {
    fn default() -> Self {
        let pattern = SequenceExpr::word_set(&["left", "right"])
            .then_whitespace()
            .t_aco("hand")
            .then_whitespace()
            .then_noun();

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for LeftRightHand {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        let space = &matched_tokens[1];

        Some(Lint {
            span: space.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith(vec!['-'])],
            message: "Use a hyphen in `left-hand` or `right-hand` when modifying a noun."
                .to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Ensures `left hand` and `right hand` are hyphenated when used as adjectives before a noun, such as in `left-hand side` or `right-hand corner`."
    }
}

#[cfg(test)]
mod tests {
    use super::LeftRightHand;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn corrects_left_hand_side() {
        assert_suggestion_result(
            "You'll see it on the left hand side.",
            LeftRightHand::default(),
            "You'll see it on the left-hand side.",
        );
    }

    #[test]
    fn corrects_right_hand_corner() {
        assert_suggestion_result(
            "It's in the right hand corner.",
            LeftRightHand::default(),
            "It's in the right-hand corner.",
        );
    }

    #[test]
    fn does_not_correct_noun_usage() {
        assert_suggestion_result(
            "She raised her right hand.",
            LeftRightHand::default(),
            "She raised her right hand.",
        );
    }
}
