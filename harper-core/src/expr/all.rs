use crate::{Span, Token, expr::Expr};

/// An [`Expr`] that matches against tokens if and only if all of its children do.
/// This can be useful for situations where you have multiple expressions that represent a grammatical
/// error, but you need _all_ of them to match to be certain.
///
/// It will return the position of the farthest window.
#[derive(Default)]
pub struct All {
    children: Vec<Box<dyn Expr>>,
}

impl All {
    pub fn new(children: Vec<Box<dyn Expr>>) -> Self {
        Self { children }
    }

    pub fn add(&mut self, e: impl Expr + 'static) {
        self.children.push(Box::new(e));
    }
}

impl Expr for All {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        let mut longest: Option<Span<Token>> = None;

        for expr in self.children.iter() {
            let window = expr.run(cursor, tokens, source)?;

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
