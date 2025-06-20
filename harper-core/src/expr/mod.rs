mod all;
mod anchor_end;
mod anchor_start;
mod expr_map;
mod first_match_of;
mod fixed_phrase;
mod longest_match_of;
mod mergeable_words;
mod reflexive_pronoun;
mod repeating;
mod sequence_expr;
mod similar_to_phrase;
mod spelled_number_expr;
mod step;
mod time_unit_expr;
mod unless_step;
mod word_expr_group;

#[cfg(not(feature = "concurrent"))]
use std::rc::Rc;
use std::sync::Arc;

pub use all::All;
pub use anchor_end::AnchorEnd;
pub use anchor_start::AnchorStart;
pub use expr_map::ExprMap;
pub use fixed_phrase::FixedPhrase;
pub use longest_match_of::LongestMatchOf;
pub use mergeable_words::MergeableWords;
pub use reflexive_pronoun::ReflexivePronoun;
pub use repeating::Repeating;
pub use sequence_expr::SequenceExpr;
pub use similar_to_phrase::SimilarToPhrase;
pub use spelled_number_expr::SpelledNumberExpr;
pub use step::Step;
pub use time_unit_expr::TimeUnitExpr;
pub use unless_step::UnlessStep;
pub use word_expr_group::WordExprGroup;

use crate::{Document, LSend, Span, Token};

/// A common problem in Harper is that we need to identify tokens that fulfil certain criterion.
/// An `Expr` is a way to express whether a certain set of tokens fulfil that criteria.
/// When supplied a specific position in a token stream, the job of an `Expr` is to determine the window of tokens (including the cursor itself) that fulfils whatever criteria the author desires.
/// It is then the job of another system to identify portions of documents that fulfil this criteria.
pub trait Expr: LSend {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span>;
}

impl<S> Expr for S
where
    S: Step + ?Sized,
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

impl<E> Expr for Arc<E>
where
    E: Expr,
{
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span> {
        self.as_ref().run(cursor, tokens, source)
    }
}

#[cfg(not(feature = "concurrent"))]
impl<E> Expr for Rc<E>
where
    E: Expr,
{
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span> {
        self.as_ref().run(cursor, tokens, source)
    }
}

fn add(u: usize, i: isize) -> Option<usize> {
    if i.is_negative() {
        u.checked_sub(i.wrapping_abs() as u32 as usize)
    } else {
        u.checked_add(i as usize)
    }
}

pub trait ExprExt {
    /// Iterate over all matches of this expression in the document, automatically filtering out
    /// overlapping matches, preferring the first.
    fn iter_matches<'a>(
        &'a self,
        tokens: &'a [Token],
        source: &'a [char],
    ) -> Box<dyn Iterator<Item = Span> + 'a>;

    fn iter_matches_in_doc<'a>(&'a self, doc: &'a Document) -> Box<dyn Iterator<Item = Span> + 'a>;
}

impl<E: ?Sized> ExprExt for E
where
    E: Expr,
{
    fn iter_matches<'a>(
        &'a self,
        tokens: &'a [Token],
        source: &'a [char],
    ) -> Box<(dyn Iterator<Item = Span> + 'a)> {
        let mut last_end = 0usize;

        Box::new((0..tokens.len()).filter_map(move |i| {
            let span = self.run(i, tokens, source)?;
            if span.start >= last_end {
                last_end = span.end;
                Some(span)
            } else {
                None
            }
        }))
    }

    fn iter_matches_in_doc<'a>(
        &'a self,
        doc: &'a Document,
    ) -> Box<(dyn Iterator<Item = Span> + 'a)> {
        Box::new(self.iter_matches(doc.get_tokens(), doc.get_source()))
    }
}

pub trait OwnedExprExt {
    fn or(self, other: impl Expr + 'static) -> LongestMatchOf;
}

impl<E> OwnedExprExt for E
where
    E: Expr + 'static,
{
    fn or(self, other: impl Expr + 'static) -> LongestMatchOf {
        LongestMatchOf::new(vec![Box::new(self), Box::new(other)])
    }
}
