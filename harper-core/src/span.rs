use std::{fmt::Display, marker::PhantomData, ops::Range};

use serde::{Deserialize, Serialize};

use crate::Token;

/// A window in a `T` sequence.
///
/// Note that the range covered by a [`Span`] is end-exclusive, meaning that the end index is not
/// included in the range covered by the [`Span`]. If you're familiar with the Rust range syntax,
/// you could say the span covers the equivalent of `start..end`, *not* `start..=end`.
///
/// For a [`Span`] to be correct, its end index must be greater than or equal to its start
/// index. Creating or using a [`Span`] which does not follow this rule may lead to unexpected
/// behavior or panics.
///
/// Although specific to `harper.js`, [this page may clear up any questions you have](https://writewithharper.com/docs/harperjs/spans).
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct Span<T> {
    /// The start index of the span.
    pub start: usize,
    /// The end index of the span.
    ///
    /// Note that [`Span`] represents an exclusive range. This means that a `Span::new(0, 5)` will
    /// cover the values `0, 1, 2, 3, 4`; it will not cover the `5`.
    pub end: usize,
    #[serde(skip)]
    span_type: PhantomData<T>,
}

impl<T> Span<T> {
    /// A [`Span`] with a start and end index of 0.
    pub const ZERO: Self = Self::empty(0);

    /// Creates a new [`Span`] with the provided start and end indices.
    ///
    /// # Panics
    ///
    /// This will panic if `start` is greater than `end`.
    pub fn new(start: usize, end: usize) -> Self {
        if start > end {
            panic!("{start} > {end}");
        }
        Self {
            start,
            end,
            span_type: PhantomData,
        }
    }

    /// Creates a new [`Span`] from the provided start position and length.
    pub fn new_with_len(start: usize, len: usize) -> Self {
        Self {
            start,
            end: start + len,
            span_type: PhantomData,
        }
    }

    /// Creates a new empty [`Span`] with the provided position.
    pub const fn empty(pos: usize) -> Self {
        Self {
            start: pos,
            end: pos,
            span_type: PhantomData,
        }
    }

    /// The length of the [`Span`].
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Checks whether the [`Span`] is empty.
    ///
    /// A [`Span`] is considered empty if it has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks whether `idx` is within the range of the span.
    pub fn contains(&self, idx: usize) -> bool {
        self.start <= idx && idx < self.end
    }

    /// Checks whether this span's range overlaps with `other`.
    pub fn overlaps_with(&self, other: Self) -> bool {
        (self.start < other.end) && (other.start < self.end)
    }

    /// Get the associated content. Will return [`None`] if the span is non-empty and any aspect is
    /// invalid.
    pub fn try_get_content<'a>(&self, source: &'a [T]) -> Option<&'a [T]> {
        if self.is_empty() {
            Some(&source[0..0])
        } else {
            source.get(self.start..self.end)
        }
    }

    /// Expand the span by either modifying [`Self::start`] or [`Self::end`] to include the target
    /// index.
    ///
    /// Does nothing if the span already includes the target.
    pub fn expand_to_include(&mut self, target: usize) {
        if target < self.start {
            self.start = target;
        } else if target >= self.end {
            self.end = target + 1;
        }
    }

    /// Get the associated content. Will panic if any aspect is invalid.
    pub fn get_content<'a>(&self, source: &'a [T]) -> &'a [T] {
        match self.try_get_content(source) {
            Some(v) => v,
            None => panic!("Failed to get content for span."),
        }
    }

    /// Set the span's length.
    pub fn set_len(&mut self, length: usize) {
        self.end = self.start + length;
    }

    /// Returns a copy of this [`Span`] with a new length.
    pub fn with_len(&self, length: usize) -> Self {
        let mut cloned = *self;
        cloned.set_len(length);
        cloned
    }

    /// Add an amount to both [`Self::start`] and [`Self::end`]
    pub fn push_by(&mut self, by: usize) {
        self.start += by;
        self.end += by;
    }

    /// Subtract an amount from both [`Self::start`] and [`Self::end`]
    pub fn pull_by(&mut self, by: usize) {
        self.start -= by;
        self.end -= by;
    }

    /// Add an amount to a copy of both [`Self::start`] and [`Self::end`]
    pub fn pushed_by(&self, by: usize) -> Self {
        let mut clone = *self;
        clone.start += by;
        clone.end += by;
        clone
    }

    /// Subtract an amount to a copy of both [`Self::start`] and [`Self::end`]
    pub fn pulled_by(&self, by: usize) -> Option<Self> {
        if by > self.start {
            return None;
        }

        let mut clone = *self;
        clone.start -= by;
        clone.end -= by;
        Some(clone)
    }
}

/// Additional functions for types that implement [`std::fmt::Debug`] and [`Display`].
impl<T: Display + std::fmt::Debug> Span<T> {
    /// Gets the content of this [`Span<T>`] as a [`String`].
    pub fn get_content_string(&self, source: &[T]) -> String {
        if let Some(content) = self.try_get_content(source) {
            content.iter().map(|t| t.to_string()).collect()
        } else {
            panic!("Could not get position {self:?} within \"{source:?}\"")
        }
    }
}

/// Functionality specific to [`Token`] spans.
impl Span<Token> {
    /// Converts the [`Span<Token>`] into a [`Span<char>`].
    ///
    /// This requires knowing the character spans of the tokens covered by this
    /// [`Span<Token>`]. Because of this, a reference to the source token sequence used to create
    /// this span is required.
    pub fn to_char_span(&self, source_document_tokens: &[Token]) -> Span<char> {
        if self.is_empty() {
            Span::ZERO
        } else {
            let target_tokens = &source_document_tokens[self.start..self.end];
            Span::new(
                target_tokens.first().unwrap().span.start,
                target_tokens.last().unwrap().span.end,
            )
        }
    }
}

impl<T> From<Range<usize>> for Span<T> {
    /// Reinterprets the provided [`std::ops::Range`] as a [`Span`].
    fn from(value: Range<usize>) -> Self {
        Self::new(value.start, value.end)
    }
}

impl<T> From<Span<T>> for Range<usize> {
    /// Converts the [`Span`] to an [`std::ops::Range`].
    fn from(value: Span<T>) -> Self {
        value.start..value.end
    }
}

impl<T> IntoIterator for Span<T> {
    type Item = usize;

    type IntoIter = Range<usize>;

    /// Converts the [`Span`] into an iterator that yields the indices covered by its range.
    ///
    /// Note that [`Span`] is half-open, meaning that the value [`Self::end`] will not be yielded
    /// by this iterator: it will stop at the index immediately preceding [`Self::end`].
    fn into_iter(self) -> Self::IntoIter {
        self.start..self.end
    }
}

impl<T> Clone for Span<T> {
    // Note: manual implementation so we don't unnecessarily require `T` to impl `Clone`.
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Span<T> {}

#[cfg(test)]
mod tests {
    use crate::{
        Document,
        expr::{ExprExt, SequenceExpr},
    };

    use super::Span;

    type UntypedSpan = Span<()>;

    #[test]
    fn overlaps() {
        assert!(UntypedSpan::new(0, 5).overlaps_with(UntypedSpan::new(3, 6)));
        assert!(UntypedSpan::new(0, 5).overlaps_with(UntypedSpan::new(2, 3)));
        assert!(UntypedSpan::new(0, 5).overlaps_with(UntypedSpan::new(4, 5)));
        assert!(UntypedSpan::new(0, 5).overlaps_with(UntypedSpan::new(4, 4)));

        assert!(!UntypedSpan::new(0, 3).overlaps_with(UntypedSpan::new(3, 5)));
    }

    #[test]
    fn expands_properly() {
        let mut span = UntypedSpan::new(2, 2);

        span.expand_to_include(1);
        assert_eq!(span, UntypedSpan::new(1, 2));

        span.expand_to_include(2);
        assert_eq!(span, UntypedSpan::new(1, 3));
    }

    #[test]
    fn to_char_span_converts_correctly() {
        let doc = Document::new_plain_english_curated("Hello world!");

        // Empty span.
        let token_span = Span::ZERO;
        let converted = token_span.to_char_span(doc.get_tokens());
        assert!(converted.is_empty());

        // Span from `Expr`.
        let token_span = SequenceExpr::any_word()
            .t_ws()
            .then_any_word()
            .iter_matches_in_doc(&doc)
            .next()
            .unwrap();
        let converted = token_span.to_char_span(doc.get_tokens());
        assert_eq!(
            converted.get_content_string(doc.get_source()),
            "Hello world"
        );
    }
}
