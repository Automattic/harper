use hashbrown::HashMap;

use super::first_match_of::FirstMatchOf;
use super::{Expr, SequenceExpr};
use crate::{CharString, Span, Token};

/// An expression collection to look for expressions that start with a specific
/// word.
///
/// The benefit of using this struct over other methods increases for larger collections.
#[derive(Default)]
pub struct WordExprGroup<E>
where
    E: Expr,
{
    exprs: HashMap<CharString, E>,
}

impl WordExprGroup<FirstMatchOf> {
    pub fn add(&mut self, word: &str, expr: impl Expr + 'static) {
        let chars = word.chars().collect();

        if let Some(group) = self.exprs.get_mut(&chars) {
            group.add(expr);
        } else {
            let mut group = FirstMatchOf::default();
            group.add(expr);
            self.exprs.insert(chars, group);
        }
    }

    /// Add a pattern that matches just a word on its own, without anything else required to match.
    pub fn add_word(&mut self, word: &'static str) {
        self.add(word, SequenceExpr::default().then_exact_word(word));
    }
}

impl<E> Expr for WordExprGroup<E>
where
    E: Expr,
{
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        let first = tokens.get(cursor)?;
        if !first.kind.is_word() {
            return None;
        }

        let word_chars = first.span.get_content(source);
        let inner_pattern = self.exprs.get(word_chars)?;

        inner_pattern.run(cursor, tokens, source)
    }
}
