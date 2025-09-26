use crate::expr::FirstMatchOf;
use crate::patterns::WhitespacePattern;
use crate::{Span, Token};

use super::Expr;

/// Matches either a space or a hyphen, useful for matching compound words.
#[derive(Default)]
pub struct SpaceOrHyphen;

impl Expr for SpaceOrHyphen {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        FirstMatchOf::new(vec![
            Box::new(WhitespacePattern),
            Box::new(|tok: &Token, _source: &[char]| tok.kind.is_hyphen()),
        ])
        .run(cursor, tokens, source)
    }
}
