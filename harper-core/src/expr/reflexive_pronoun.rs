use crate::{Span, Token, expr::Expr, patterns::WordSet};

#[derive(Default)]
pub struct ReflexivePronoun;

impl Expr for ReflexivePronoun {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span> {
        let expr = WordSet::new(&[
            "herself",
            "himself",
            "itself",
            "myself",
            "oneself",
            "oneselves",
            "ourself",
            "ourselves",
            "theirself",
            "theirselves",
            "themself",
            "themselves",
            "thyself",
            "yourself",
            "yourselves",
        ]);
        expr.run(cursor, tokens, source)
    }
}
