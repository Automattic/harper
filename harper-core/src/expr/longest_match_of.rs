use crate::{Span, Token, expr::Expr};

/// An [`Expr`] that returns the farthest offset of the longest match in a list of expressions.
#[derive(Default)]
pub struct LongestMatchOf {
    exprs: Vec<Box<dyn Expr>>,
}

impl LongestMatchOf {
    pub fn new(exprs: Vec<Box<dyn Expr>>) -> Self {
        Self { exprs }
    }

    pub fn add(&mut self, expr: impl Expr + 'static) {
        self.exprs.push(Box::new(expr));
    }
}

impl Expr for LongestMatchOf {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        self.exprs
            .iter()
            .filter_map(|expr| expr.run(cursor, tokens, source))
            .max_by_key(Span::len)
    }
}
