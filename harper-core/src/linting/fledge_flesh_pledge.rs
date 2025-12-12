use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct FledgeFleshPledge {
    expr: Box<dyn Expr>,
}

impl Default for FledgeFleshPledge {
    fn default() -> Self {
        Self {
            expr: Box::new(SequenceExpr::any_of(vec![
                Box::new(
                    SequenceExpr::word_set(&["full", "fully"])
                        .t_ws_h()
                        .then_word_set(&["fledged", "fleshed", "pledged"]),
                ),
                Box::new(
                    SequenceExpr::word_set(&["fledged", "fleshed", "pledged"])
                        .t_ws_h()
                        .t_aco("out"),
                ),
                Box::new(
                    SequenceExpr::word_set(&["fledge", "fledges", "fledging"])
                        .t_ws_h()
                        .t_aco("out"),
                ),
                Box::new(
                    SequenceExpr::word_set(&["flesh", "fleshes", "fleshing"])
                        .t_ws_h()
                        .t_aco("out"),
                ),
                Box::new(
                    SequenceExpr::word_set(&["pledge", "pledges", "pledging"])
                        .t_ws_h()
                        .t_aco("out"),
                ),
            ])),
        }
    }
}

impl ExprLinter for FledgeFleshPledge {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        format_lint_match(toks, ctx, src);
        None
    }

    fn description(&self) -> &str {
        "Fledge vs flesh vs pledge"
    }
}
