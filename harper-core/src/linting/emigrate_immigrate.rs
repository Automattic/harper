use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, expr_linter::Chunk},
};

pub struct EmigrateImmigrate {
    expr: Box<dyn Expr>,
}

impl Default for EmigrateImmigrate {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&[
                    "emigrate",
                    "immigrate",
                    "emigrated",
                    "immigrated",
                    "emigrates",
                    "immigrates",
                    "emigrating",
                    "immigrating",
                    "emigration",
                    "immigration",
                ])
                .t_ws()
                .then_preposition(),
            ),
        }
    }
}

impl ExprLinter for EmigrateImmigrate {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks.span()?,
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects "
    }
}
