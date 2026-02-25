use crate::Token;
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::patterns::InflectionOfBe;

use super::expr_linter::Chunk;
use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct FindFine {
    expr: Box<dyn Expr>,
}

impl Default for FindFine {
    fn default() -> Self {
        let expr = SequenceExpr::with(InflectionOfBe::default())
            .t_ws()
            .t_aco("find");

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for FindFine {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_word = matched_tokens.get(2)?;

        Some(Lint {
            span: offending_word.span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "fine",
                offending_word.span.get_content(source),
            )],
            message: "Did you mean `fine`?".to_owned(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Fixes the common typo where writers write `find` when they mean `fine`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::FindFine;

    #[test]
    fn issue_2115() {
        assert_suggestion_result(
            "I was using oil.nvim from an year and everything was find for me but I was missing a very key feature",
            FindFine::default(),
            "I was using oil.nvim from an year and everything was fine for me but I was missing a very key feature",
        );
        assert_suggestion_result(
            "I made several observations throughout the evening and everything was find.",
            FindFine::default(),
            "I made several observations throughout the evening and everything was fine.",
        );
        assert_suggestion_result(
            "I am find not using GPU at all for open3d.",
            FindFine::default(),
            "I am fine not using GPU at all for open3d.",
        );
    }
}
