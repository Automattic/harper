use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct SomeWithoutArticle {
    expr: Box<dyn Expr>,
}

impl Default for SomeWithoutArticle {
    fn default() -> Self {
        let expr = SequenceExpr::any_capitalization_of("the")
            .t_ws()
            .then_any_capitalization_of("some");

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for SomeWithoutArticle {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let template = span.get_content(source);
        let some_chars = matched_tokens.last()?.span.get_content(source);

        let suggestions = vec![
            Suggestion::ReplaceWith(some_chars.to_vec()),
            Suggestion::replace_with_match_case("the same".chars().collect(), template),
        ];

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            message:
                "Use `some` on its own here, or switch to `the same` if that was the intention."
                    .to_owned(),
            suggestions,
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects the redundant article in front of `some` and suggests more natural phrasing."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_lint_count, assert_nth_suggestion_result, assert_suggestion_result,
    };

    use super::SomeWithoutArticle;

    #[test]
    fn fixes_simple_lowercase() {
        assert_suggestion_result(
            "We interviewed the some candidates today.",
            SomeWithoutArticle::default(),
            "We interviewed some candidates today.",
        );
    }

    #[test]
    fn fixes_sentence_case() {
        assert_suggestion_result(
            "The Some volunteers arrived early.",
            SomeWithoutArticle::default(),
            "Some volunteers arrived early.",
        );
    }

    #[test]
    fn preserves_uppercase_block() {
        assert_suggestion_result(
            "THE SOME OPTIONS WERE LISTED.",
            SomeWithoutArticle::default(),
            "SOME OPTIONS WERE LISTED.",
        );
    }

    #[test]
    fn second_suggestion_produces_the_same() {
        assert_nth_suggestion_result(
            "We kept the some approach from last year.",
            SomeWithoutArticle::default(),
            "We kept the same approach from last year.",
            1,
        );
    }

    #[test]
    fn ignores_already_correct_some() {
        assert_lint_count(
            "We interviewed some candidates today.",
            SomeWithoutArticle::default(),
            0,
        );
    }

    #[test]
    fn ignores_the_same() {
        assert_lint_count(
            "We kept the same approach from last year.",
            SomeWithoutArticle::default(),
            0,
        );
    }

    #[test]
    fn ignores_the_something() {
        assert_lint_count(
            "We interviewed the something else entirely.",
            SomeWithoutArticle::default(),
            0,
        );
    }

    #[test]
    fn works_before_comma() {
        assert_suggestion_result(
            "They reviewed the some, then finalized the list.",
            SomeWithoutArticle::default(),
            "They reviewed some, then finalized the list.",
        );
    }

    #[test]
    fn works_before_possessive_noun() {
        assert_suggestion_result(
            "The report praised the some team's effort.",
            SomeWithoutArticle::default(),
            "The report praised some team's effort.",
        );
    }

    #[test]
    fn handles_line_break_spacing() {
        assert_suggestion_result(
            "We invited the some\nartists to perform.",
            SomeWithoutArticle::default(),
            "We invited some\nartists to perform.",
        );
    }
}
