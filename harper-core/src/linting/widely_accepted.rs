use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct WidelyAccepted {
    expr: SequenceExpr,
}

impl Default for WidelyAccepted {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .t_aco("wide")
            .then_whitespace()
            .then_word_set(&["accepted", "acceptable", "used"]);

        Self { expr }
    }
}

impl ExprLinter for WidelyAccepted {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        // We only need to replace the `wide` token with `widely`.
        let wide_token = matched_tokens.first()?;
        let wide_chars = wide_token.span.get_content(source);

        Some(Lint {
            span: wide_token.span,
            lint_kind: LintKind::Miscellaneous,
            message: "Use the adverb `widely` in this context. For example, `widely accepted` or `widely used` is standard usage."
                .to_owned(),
            suggestions: vec![Suggestion::replace_with_match_case_str("widely", wide_chars)],
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Flags `wide accepted`, `wide acceptable`, or `wide used` and recommends switching `wide` to the adverb `widely`."
    }
}

#[cfg(test)]
mod tests {
    use super::WidelyAccepted;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn wide_accepted_lowercase() {
        assert_suggestion_result(
            "It is wide accepted that exercise improves health.",
            WidelyAccepted::default(),
            "It is widely accepted that exercise improves health.",
        );
    }

    #[test]
    fn wide_acceptable_mixed_case() {
        assert_suggestion_result(
            "Wide acceptable standards are used in the design.",
            WidelyAccepted::default(),
            "Widely acceptable standards are used in the design.",
        );
    }

    #[test]
    fn widely_already_correct() {
        assert_lint_count(
            "It is widely accepted that sunlight is beneficial in moderation.",
            WidelyAccepted::default(),
            0,
        );
    }

    #[test]
    fn no_false_positive() {
        assert_lint_count(
            "The house had wide open windows during the renovation.",
            WidelyAccepted::default(),
            0,
        );
    }

    #[test]
    fn wide_accepted_in_long_text() {
        assert_suggestion_result(
            "This is an example paragraph, and it is wide accepted that these changes will improve performance. In fact, widely used frameworks have already adopted them.",
            WidelyAccepted::default(),
            "This is an example paragraph, and it is widely accepted that these changes will improve performance. In fact, widely used frameworks have already adopted them.",
        );
    }

    #[test]
    fn wide_twice_in_one_sentence() {
        assert_suggestion_result(
            "It is wide accepted and wide used by many professionals.",
            WidelyAccepted::default(),
            "It is widely accepted and widely used by many professionals.",
        );
    }
}
