use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

/// Both `another thing coming` and `another think coming` are correct, but `another think coming` is the original.
pub struct AnotherThinkComing {
    expr: Box<dyn Expr>,
}

impl Default for AnotherThinkComing {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["had", "has", "have", "got"])
                    .then_fixed_phrase(" another thing coming"),
            ),
        }
    }
}

impl ExprLinter for AnotherThinkComing {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks[2..].span()?,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "another think coming",
                toks.span()?.get_content(src),
            )],
            message: "Corrects `another thing coming` to `another think coming`".to_string(),
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Though `another thing coming` is now more common, `another think coming` is the original phrase."
    }
}

#[cfg(test)]
pub mod tests {
    use super::AnotherThinkComing;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn fix_got_another_thing_coming() {
        assert_suggestion_result(
            "If Microsoft thinks my Team and I are going to REINSTALL Windows fresh on over 1500 PC's they've got another thing coming!!",
            AnotherThinkComing::default(),
            "If Microsoft thinks my Team and I are going to REINSTALL Windows fresh on over 1500 PC's they've got another think coming!!",
        );
    }

    #[test]
    fn fix_has_another_thing_coming() {
        assert_suggestion_result(
            "Anyone who thinks it's easy to raise a child has another thing coming.",
            AnotherThinkComing::default(),
            "Anyone who thinks it's easy to raise a child has another think coming.",
        );
    }

    #[test]
    fn fix_have_another_thing_coming() {
        assert_suggestion_result(
            "And if you think they're predictable, you have another thing coming still.",
            AnotherThinkComing::default(),
            "And if you think they're predictable, you have another think coming still.",
        );
    }

    #[test]
    fn fix_had_another_thing_coming() {
        assert_suggestion_result(
            "And wouldn't you know it I had another thing coming.",
            AnotherThinkComing::default(),
            "And wouldn't you know it I had another think coming.",
        );
    }
}
