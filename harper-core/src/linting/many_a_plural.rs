use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct ManyAPlural {
    expr: SequenceExpr,
}

impl Default for ManyAPlural {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("many")
                .t_ws()
                .then_indefinite_article()
                .t_ws()
                .then_plural_noun(),
        }
    }
}

impl ExprLinter for ManyAPlural {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], _source: &[char]) -> Option<Lint> {
        // 2 fixes
        // 1. "many a" should be "many" - DONE
        // 2. "plural noun" should be "plural nouns" - TODO

        Some(Lint {
            span: toks[1..3].span()?,
            lint_kind: LintKind::Agreement,
            suggestions: vec![Suggestion::Remove],
            message: "Drop the indefinite article `a` or make the noun singular.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter skeleton for contributors to copy into `harper_core/src/linting/` and rename."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::ManyAPlural;

    #[test]
    fn times() {
        assert_suggestion_result(
            "Many a times have I debugged an issue and was convinced of where is the bug and what the fix is.",
            ManyAPlural::default(),
            "Many times have I debugged an issue and was convinced of where is the bug and what the fix is.",
        );
    }

    #[test]
    fn people() {
        assert_suggestion_result(
            "Many a people play games with gamepad.",
            ManyAPlural::default(),
            "Many people play games with gamepad.",
        );
    }
}
