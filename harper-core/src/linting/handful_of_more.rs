use crate::{
    Lint, Token, TokenKind, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct HandfulOfMore {
    expr: SequenceExpr,
}

impl Default for HandfulOfMore {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("handful")
                .t_ws()
                .t_aco("of")
                .t_ws()
                .t_aco("more")
                .t_ws()
                .then_kind_is_but_is_not(TokenKind::is_noun, TokenKind::is_adjective),
        }
    }
}

impl ExprLinter for HandfulOfMore {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        let (first_ws_idx, of_idx) = (1, 2);

        Some(Lint {
            span: matched_tokens[first_ws_idx..=of_idx].span()?,
            lint_kind: LintKind::Nonstandard,
            suggestions: vec![Suggestion::Remove],
            message: "Using `of` in this construction is not standard.".to_string(),
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
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::HandfulOfMore;

    #[test]
    fn fix_handful_of_more_times() {
        assert_suggestion_result(
            "(and possibly a handful of more times as needed to resolve any issues)",
            HandfulOfMore::default(),
            "(and possibly a handful more times as needed to resolve any issues)",
        );
    }

    #[test]
    fn fix_handful_of_more_prs() {
        assert_suggestion_result(
            "(maybe as soon as tomorrow, but trying to get a handful of more PRs merged...)",
            HandfulOfMore::default(),
            "(maybe as soon as tomorrow, but trying to get a handful more PRs merged...)",
        );
    }

    #[test]
    fn dont_flag_comparative() {
        assert_no_lints(
            "After writing a handful of more advanced recipes I often found myself running into situations where the context wasn't recognized",
            HandfulOfMore::default(),
        );
    }
}
