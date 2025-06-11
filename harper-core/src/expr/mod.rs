mod all;
mod condition;
mod longest_match_of;
mod sequence_expr;
mod step;

pub use all::All;
pub use condition::Condition;
pub use longest_match_of::LongestMatchOf;
pub use sequence_expr::SequenceExpr;
pub use step::Step;

use crate::{LSend, Span, Token};

pub trait Expr: LSend {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span>;
}

impl<S> Expr for S
where
    S: Step,
{
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span> {
        self.step(tokens, cursor, source).map(|s| {
            if s >= 0 {
                Span::new_with_len(cursor, s as usize)
            } else {
                Span::new(add(cursor, s).unwrap(), cursor)
            }
        })
    }
}

fn add(u: usize, i: isize) -> Option<usize> {
    if i.is_negative() {
        u.checked_sub(i.wrapping_abs() as u32 as usize)
    } else {
        u.checked_add(i as usize)
    }
}
