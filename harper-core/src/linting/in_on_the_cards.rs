use crate::{expr::{Expr, SequenceExpr}, Token};

use super::{ExprLinter, Lint};

pub struct InOnTheCards {
    expr: Box<dyn Expr>,
}

impl Default for InOnTheCards {
    fn default() -> Self {
        Self {
            expr: Box::new(SequenceExpr::aco("in").then_whitespace().t_aco("the").then_whitespace().t_aco("cards")),
        }
    }
}

impl ExprLinter for InOnTheCards {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }
    
    fn match_to_lint(&self,toks: &[Token], src: &[char]) -> Option<Lint> {
        None
    }
    
    fn description(&self) ->  &str {
        "Corrects either `in the cards` or `on the cards` to the other, depending on the dialect."
    }
}
