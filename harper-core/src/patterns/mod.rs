//! [`Pattern`]s are one of the more powerful ways to query text inside Harper, especially for beginners.
//!
//! Through the [`PatternLinter`](crate::linting::PatternLinter) trait, they make it much easier to
//! build Harper [rules](crate::linting::Linter).
//!
//! See the page about [`SequencePattern`] for a concrete example of their use.

use std::{collections::VecDeque, num::NonZeroUsize};

use crate::{Document, Span, Token, VecExt};

mod all;
mod any_capitalization;
mod any_pattern;
mod either_pattern;
mod exact_phrase;
mod implies_quantity;
mod indefinite_article;
mod invert;
mod naive_pattern_group;
mod nominal_phrase;
mod pattern_map;
mod repeating_pattern;
mod sequence_pattern;
mod similar_to_phrase;
mod split_compound_word;
mod whitespace_pattern;
mod within_edit_distance;
mod word_pattern_group;
mod word_set;

pub use all::All;
pub use any_capitalization::AnyCapitalization;
pub use any_pattern::AnyPattern;
use blanket::blanket;
pub use either_pattern::EitherPattern;
pub use exact_phrase::ExactPhrase;
pub use implies_quantity::ImpliesQuantity;
pub use indefinite_article::IndefiniteArticle;
pub use invert::Invert;
pub use naive_pattern_group::NaivePatternGroup;
pub use nominal_phrase::NominalPhrase;
pub use pattern_map::PatternMap;
pub use repeating_pattern::RepeatingPattern;
pub use sequence_pattern::SequencePattern;
pub use similar_to_phrase::SimilarToPhrase;
pub use split_compound_word::SplitCompoundWord;
pub use whitespace_pattern::WhitespacePattern;
pub use word_pattern_group::WordPatternGroup;
pub use word_set::WordSet;

#[cfg(not(feature = "concurrent"))]
#[blanket(derive(Rc, Arc))]
pub trait Pattern {
    /// Check if the pattern matches at the start of the given token slice.
    ///
    /// Returns the length of the match if successful, or `None` if not.
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize>;
}
#[cfg(feature = "concurrent")]
#[blanket(derive(Arc))]
pub trait Pattern: Send + Sync {
    /// Check if the pattern matches at the start of the given token slice.
    ///
    /// Returns the length of the match if successful, or `None` if not.
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize>;
}

pub trait PatternExt {
    /// Search through all tokens to locate all non-overlapping pattern matches.
    fn find_all_matches(&self, tokens: &[Token], source: &[char]) -> Vec<Span>;
}

impl<P> PatternExt for P
where
    P: Pattern,
{
    fn find_all_matches(&self, tokens: &[Token], source: &[char]) -> Vec<Span> {
        let mut found = Vec::new();

        for i in 0..tokens.len() {
            let len = self.matches(&tokens[i..], source);

            if let Some(len) = len {
                found.push(Span::new_with_len(i, len.get()));
            }
        }

        if found.len() < 2 {
            return found;
        }

        let mut remove_indices = VecDeque::new();

        for i in 0..found.len() - 1 {
            let cur = &found[i];
            let next = &found[i + 1];

            if cur.overlaps_with(*next) {
                remove_indices.push_back(i + 1);
            }
        }

        found.remove_indices(remove_indices);

        found
    }
}

pub trait OwnedPatternExt {
    fn or(self, other: Box<dyn Pattern>) -> EitherPattern;
}

impl<P> OwnedPatternExt for P
where
    P: Pattern + 'static,
{
    fn or(self, other: Box<dyn Pattern>) -> EitherPattern {
        EitherPattern::new(vec![Box::new(self), other])
    }
}

#[cfg(feature = "concurrent")]
impl<F> Pattern for F
where
    F: Fn(&Token, &[char]) -> bool,
    F: Send + Sync,
{
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        NonZeroUsize::new(if self(tokens.first()?, source) { 1 } else { 0 })
    }
}

#[cfg(not(feature = "concurrent"))]
impl<F> Pattern for F
where
    F: Fn(&Token, &[char]) -> bool,
{
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        NonZeroUsize::new(if self(tokens.first()?, source) { 1 } else { 0 })
    }
}

pub trait DocPattern {
    fn find_all_matches_in_doc(&self, document: &Document) -> Vec<Span>;
}

impl<P: PatternExt> DocPattern for P {
    fn find_all_matches_in_doc(&self, document: &Document) -> Vec<Span> {
        self.find_all_matches(document.get_tokens(), document.get_source())
    }
}
