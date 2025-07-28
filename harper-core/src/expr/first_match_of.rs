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
    pub fn new(exprs: Vec<Box<dyn Expr>>) -> Self {
        Self { exprs }
    }

    pub fn add(&mut self, expr: impl Expr + 'static) {
        self.exprs.push(Box::new(expr));
    }
}

impl Expr for FirstMatchOf {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        self.exprs
            .iter()
            .find_map(|p| p.run(cursor, tokens, source))
    }
}
