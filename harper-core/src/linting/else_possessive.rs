use crate::expr::Expr;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct ElsePossessive {
    expr: Box<dyn Expr>,
}

impl Default for ElsePossessive {
    fn default() -> Self {
        let pronouns = WordSet::new(&[
            "somebody",
            "someone",
            "anybody",
            "anyone",
            "everybody",
            "everyone",
            "nobody",
        ])
        .or(SequenceExpr::aco("no").then_whitespace().t_aco("one"));

        let pattern = SequenceExpr::with(pronouns)
            .then_whitespace()
            .t_aco("elses");

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for ElsePossessive {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let offender = toks.last()?;
        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith("else's".chars().collect())],
            message: "Add the missing possessive apostrophe: use `else’s`.".to_owned(),
            priority: 60,
        })
    }

    fn description(&self) -> &str {
        "Detects missing apostrophes in phrases like `someone elses book` and suggests the correct possessive form `else’s`."
    }
}

#[cfg(test)]
mod tests {
    use super::ElsePossessive;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fixes_no_one_elses() {
        assert_suggestion_result(
            "It's no one elses problem.",
            ElsePossessive::default(),
            "It's no one else's problem.",
        );
    }

    #[test]
    fn fixes_someone_elses() {
        assert_suggestion_result(
            "It's someone elses problem.",
            ElsePossessive::default(),
            "It's someone else's problem.",
        );
    }

    #[test]
    fn fixes_anybody_elses() {
        assert_suggestion_result(
            "Was that anybody elses idea?",
            ElsePossessive::default(),
            "Was that anybody else's idea?",
        );
    }

    #[test]
    fn fixes_everyone_elses() {
        assert_suggestion_result(
            "He echoed everyone elses concerns.",
            ElsePossessive::default(),
            "He echoed everyone else's concerns.",
        );
    }

    #[test]
    fn ignores_correct_form() {
        assert_lint_count(
            "She borrowed someone else's notes.",
            ElsePossessive::default(),
            0,
        );
    }
}
