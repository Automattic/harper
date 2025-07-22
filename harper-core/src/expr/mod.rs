//! An `Expr` is a declarative way to express whether a certain set of tokens fulfill a criteria.
//!
//! For example, if we want to look for the word "that" followed by an adjective, we could build an
//! expression to do so.
//!
//! The actual searching is done by another system (usually a part of the [lint framework](crate::linting::ExprLinter)).
//! It iterates through a document, checking if each index matches the criteria.
//!
//! When supplied a specific position in a token stream, the technical job of an `Expr` is to determine the window of tokens (including the cursor itself) that fulfills whatever criteria the author desires.
//!
//! The goal of the `Expr` initiative is to make rules easier to _read_ as well as to write.
//! Gone are the days of trying to manually parse the logic of another man's Rust code.
//!
//! See also: [`SequenceExpr`].

mod all;
mod anchor_end;
mod anchor_start;
mod expr_map;
mod first_match_of;
mod fixed_phrase;
mod longest_match_of;
mod mergeable_words;
mod optional;
mod reflexive_pronoun;
mod repeating;
mod sequence_expr;
mod similar_to_phrase;
mod space_or_hyphen;
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
pub use first_match_of::FirstMatchOf;
pub use fixed_phrase::FixedPhrase;
pub use longest_match_of::LongestMatchOf;
pub use mergeable_words::MergeableWords;
pub use optional::Optional;
pub use reflexive_pronoun::ReflexivePronoun;
pub use repeating::Repeating;
pub use sequence_expr::SequenceExpr;
pub use similar_to_phrase::SimilarToPhrase;
pub use space_or_hyphen::SpaceOrHyphen;
pub use spelled_number_expr::SpelledNumberExpr;
pub use step::Step;
pub use time_unit_expr::TimeUnitExpr;
pub use unless_step::UnlessStep;
pub use word_expr_group::WordExprGroup;

use crate::{Document, LSend, Span, Token};

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
    fn or(self, other: impl Expr + 'static) -> FirstMatchOf;
    fn or_longest(self, other: impl Expr + 'static) -> LongestMatchOf;
    fn and(self, other: impl Expr + 'static) -> All;
    fn and_not(self, other: impl Expr + 'static) -> All;
}

impl<E> OwnedExprExt for E
where
    E: Expr + 'static,
{
    /// Returns an expression that matches either the current one or the expression contained in `other`.
    fn or(self, other: impl Expr + 'static) -> FirstMatchOf {
        FirstMatchOf::new(vec![Box::new(self), Box::new(other)])
    }

    /// Returns an expression that matches the longest of the current one or the expression contained in `other`.
    ///
    /// If you don't need the longest match, prefer using the short-circuiting [`Self::or()`] instead.
    fn or_longest(self, other: impl Expr + 'static) -> LongestMatchOf {
        LongestMatchOf::new(vec![Box::new(self), Box::new(other)])
    }

    fn and(self, other: impl Expr + 'static) -> All {
        All::new(vec![Box::new(self), Box::new(other)])
    }

    fn and_not(self, other: impl Expr + 'static) -> All {
        self.and(UnlessStep::new(other, |_: &Token, _: &[char]| true))
    }
}
