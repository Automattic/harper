use crate::{Token, expr::Expr};

use super::Step;

/// Provides the ability to use an expression as a condition.
/// If the condition does __not match__, it will return the result of the provided step.
pub struct UnlessStep<E: Expr, S: Step> {
    condition: E,
    step: S,
}

impl<E, S> UnlessStep<E, S>
where
    E: Expr,
    S: Step,
{
    pub fn new(condition: E, step: S) -> Self {
        Self { condition, step }
    }
}

impl<E: Expr, S: Step> Step for UnlessStep<E, S> {
    fn step(&self, tokens: &[Token], cursor: usize, source: &[char]) -> Option<isize> {
        if self.condition.run(cursor, tokens, source).is_none() {
            self.step.step(tokens, cursor, source)
        } else {
            None
        }
    }
}
