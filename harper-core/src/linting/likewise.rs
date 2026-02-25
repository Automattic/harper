use crate::expr::All;
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct Likewise {
    expr: Box<dyn Expr>,
}
impl Default for Likewise {
    fn default() -> Self {
        let mut expr = All::default();

        expr.add(SequenceExpr::aco("like").then_whitespace().t_aco("wise"));
        expr.add(SequenceExpr::unless(
            SequenceExpr::anything()
                .then_whitespace()
                .then_anything()
                .then_whitespace()
                .then_noun(),
        ));

        Self {
            expr: Box::new(expr),
        }
    }
}
impl ExprLinter for Likewise {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let orig_chars = span.get_content(source);
        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "likewise".chars().collect(),
                orig_chars,
            )],
            message: format!("Did you mean the closed compound `{}`?", "likewise"),
            ..Default::default()
        })
    }
    fn description(&self) -> &'static str {
        "Looks for incorrect spacing inside the closed compound `likewise`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Likewise;

    #[test]
    fn wise_men() {
        assert_suggestion_result(
            "Like wise men, we waited.",
            Likewise::default(),
            "Like wise men, we waited.",
        );
    }

    #[test]
    fn like_wise() {
        assert_suggestion_result(
            "He acted, like wise, without hesitation.",
            Likewise::default(),
            "He acted, likewise, without hesitation.",
        );
    }
}
