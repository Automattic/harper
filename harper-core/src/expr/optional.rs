use crate::{Span, Token};

use super::Expr;

/// An optional expression.
/// Forces the optional expression to always return Some by transmuting `None` into
/// `Some(cursor..cursor)`.
pub struct Optional {
    inner: Box<dyn Expr>,
}

impl Optional {
    pub fn new(inner: impl Expr + 'static) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }
}

impl Expr for Optional {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        let res = self.inner.run(cursor, tokens, source);

        if res.is_none() {
            Some(Span::new_with_len(cursor, 0))
        } else {
            res
        }
    }
}
