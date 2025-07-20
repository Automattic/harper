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
        let mut longest: Option<Span<Token>> = None;

        for expr in self.exprs.iter() {
            let Some(window) = expr.run(cursor, tokens, source) else {
                continue;
            };

            if let Some(longest_window) = longest {
                if window.len() > longest_window.len() {
                    longest = Some(window);
                }
            } else {
                longest = Some(window);
            }
        }

        longest
    }
}
