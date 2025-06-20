use super::Expr;
use crate::{Span, Token};

/// A naive expr collection that naively iterates through a list of patterns,
/// returning the first one that matches.
///
/// Compare to [`LongestMatchOf`](super::LongestMatchOf), which returns the longest match.
#[derive(Default)]
pub struct FirstMatchOf {
    exprs: Vec<Box<dyn Expr>>,
}

impl FirstMatchOf {
    pub fn push(&mut self, expr: Box<dyn Expr>) {
        self.exprs.push(expr);
    }
}

impl Expr for FirstMatchOf {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span> {
        self.exprs
            .iter()
            .filter_map(|p| p.run(cursor, tokens, source))
            .next()
    }
}
