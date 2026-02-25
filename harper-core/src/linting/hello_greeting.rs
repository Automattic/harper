use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    expr::{AnchorStart, Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct HelloGreeting {
    expr: Box<dyn Expr>,
}

impl Default for HelloGreeting {
    fn default() -> Self {
        let expr = SequenceExpr::with(AnchorStart)
            .then_optional(SequenceExpr::default().t_ws())
            .then_optional(
                SequenceExpr::default()
                    .then_quote()
                    .then_optional(SequenceExpr::default().t_ws()),
            )
            .t_aco("halo");

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for HelloGreeting {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let word = matched_tokens.iter().find(|tok| tok.kind.is_word())?;
        let span = word.span;
        let original = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "hello".chars().collect(),
                original,
            )],
            message: "Prefer `hello` as a greeting; `halo` refers to the optical effect."
                .to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Encourages greeting someone with `hello` instead of the homophone `halo`."
    }
}

#[cfg(test)]
mod tests {
    use super::HelloGreeting;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_basic_greeting() {
        assert_suggestion_result("Halo John!", HelloGreeting::default(), "Hello John!");
    }

    #[test]
    fn corrects_with_comma() {
        assert_suggestion_result("Halo, Jane.", HelloGreeting::default(), "Hello, Jane.");
    }

    #[test]
    fn corrects_with_world() {
        assert_suggestion_result("Halo world!", HelloGreeting::default(), "Hello world!");
    }

    #[test]
    fn corrects_without_punctuation() {
        assert_suggestion_result(
            "Halo there friend.",
            HelloGreeting::default(),
            "Hello there friend.",
        );
    }

    #[test]
    fn corrects_single_word_sentence() {
        assert_suggestion_result("Halo!", HelloGreeting::default(), "Hello!");
    }

    #[test]
    fn corrects_question() {
        assert_suggestion_result("Halo?", HelloGreeting::default(), "Hello?");
    }

    #[test]
    fn corrects_uppercase() {
        assert_suggestion_result("HALO!", HelloGreeting::default(), "HELLO!");
    }

    #[test]
    fn no_lint_for_optical_term() {
        assert_lint_count(
            "The halo around the moon glowed softly.",
            HelloGreeting::default(),
            0,
        );
    }

    #[test]
    fn no_lint_mid_sentence() {
        assert_lint_count(
            "They shouted hello, not Halo, during rehearsal.",
            HelloGreeting::default(),
            0,
        );
    }

    #[test]
    fn corrects_in_quotes() {
        assert_suggestion_result(
            "\"Halo John!\"",
            HelloGreeting::default(),
            "\"Hello John!\"",
        );
    }
}
