use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::patterns::ModalVerb;
use crate::{Token, TokenStringExt};

use super::Suggestion;
use super::{ExprLinter, Lint, LintKind};
use crate::linting::expr_linter::Chunk;

pub struct DoubleModal {
    expr: Box<dyn Expr>,
}

impl Default for DoubleModal {
    fn default() -> Self {
        let expr = SequenceExpr::with(ModalVerb::default())
            .t_ws()
            .then(ModalVerb::default());

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for DoubleModal {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let first_chars = matched_tokens.first()?.span.get_content(source);
        let second_chars = matched_tokens.last()?.span.get_content(source);

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![
                Suggestion::ReplaceWith(first_chars.into()),
                Suggestion::ReplaceWith(second_chars.into()),
            ],
            message: "Two modal verbs in a row are rarely grammatical; remove one.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Two modal verbs in a row are rarely grammatical; remove one of them."
    }
}

#[cfg(test)]
mod tests {
    use super::DoubleModal;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn detects_might_could() {
        assert_lint_count(
            "They might could finish the project by Friday.",
            DoubleModal::default(),
            1,
        );
    }

    #[test]
    fn detects_should_ought() {
        assert_lint_count("You should ought to apologize.", DoubleModal::default(), 1);
    }

    #[test]
    fn allows_single_modal() {
        assert_lint_count("She must leave early.", DoubleModal::default(), 0);
    }

    #[test]
    fn detects_two_double_modals() {
        assert_lint_count(
            "He may can join us, and you might could too.",
            DoubleModal::default(),
            2,
        );
    }

    #[test]
    fn suggests_removing_second_modal_keeps_first() {
        assert_suggestion_result(
            "They might could finish the project by Friday.",
            DoubleModal::default(),
            "They might finish the project by Friday.",
        );
    }

    #[test]
    fn suggests_removing_second_modal_keeps_first_variant_order() {
        assert_suggestion_result(
            "You could might want to double-check that.",
            DoubleModal::default(),
            "You could want to double-check that.",
        );
    }

    #[test]
    fn suggests_removing_second_modal_keeps_first_capitalised() {
        assert_suggestion_result(
            "We Must Should be consistent.",
            DoubleModal::default(),
            "We Must be consistent.",
        );
    }

    #[test]
    fn allows_will_need() {
        assert_no_lints(
            "You will need administrator or editor level access",
            DoubleModal::default(),
        );
    }
}
