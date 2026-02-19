use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

/// Both `another thing coming` and `another think coming` are correct, but `another think coming` is more common.
pub struct AnotherThingComing {
    expr: Box<dyn Expr>,
}

impl Default for AnotherThingComing {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["had", "has", "have", "got"])
                    .then_fixed_phrase(" another think coming"),
            ),
        }
    }
}

impl ExprLinter for AnotherThingComing {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks[2..].span()?,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "another thing coming",
                toks.span()?.get_content(src),
            )],
            message: "Corrects `another think coming` to `another thing coming`".to_string(),
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Though `another think coming` is the original phrase, `another thing coming` is now more common."
    }
}

#[cfg(test)]
pub mod tests {
    use super::AnotherThingComing;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn fix_have_another_think_coming() {
        assert_suggestion_result(
            "If you think that, you have another think coming, English.",
            AnotherThingComing::default(),
            "If you think that, you have another thing coming, English.",
        );
    }

    #[test]
    fn fix_has_another_think_coming() {
        assert_suggestion_result(
            "If the wage earner thinks that he will obtain anything from either of the old parties he has another think coming.",
            AnotherThingComing::default(),
            "If the wage earner thinks that he will obtain anything from either of the old parties he has another thing coming.",
        );
    }

    #[test]
    #[ignore = "A lettercase bug results in 'another thiNG COMing.'"]
    fn fix_got_another_think_coming() {
        assert_suggestion_result(
            "The correct phrase is, “You've got another THINK coming.”",
            AnotherThingComing::default(),
            "The correct phrase is, “You've got another THING coming.”",
        );
    }

    #[test]
    fn fix_had_another_think_coming() {
        assert_suggestion_result(
            "Guess I had another think coming.",
            AnotherThingComing::default(),
            "Guess I had another thing coming.",
        );
    }
}
