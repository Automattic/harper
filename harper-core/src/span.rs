use std::{fmt::Display, marker::PhantomData, ops::Range};

use serde::{Deserialize, Serialize};

use crate::Token;

/// A window in a [`T`] sequence.
///
/// Although specific to `harper.js`, [this page may clear up any questions you have](https://writewithharper.com/docs/harperjs/spans).
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Span<T> {
    pub start: usize,
    pub end: usize,
    span_type: PhantomData<T>,
}

impl<T> Span<T> {
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

    pub fn new_with_len(start: usize, len: usize) -> Self {
        Self {
            start,
            end: start + len,
            span_type: PhantomData,
        }
    }

    /// Creates an empty span.
    pub fn empty() -> Self {
        Self {
            start: 0,
            end: 0,
            span_type: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, idx: usize) -> bool {
        assert!(self.start <= self.end);

        self.start <= idx && idx < self.end
    }

    pub fn overlaps_with(&self, other: Self) -> bool {
        (self.start < other.end) && (other.start < self.end)
    }

    /// Get the associated content. Will return [`None`] if any aspect is
    /// invalid.
    pub fn try_get_content<'a>(&self, source: &'a [T]) -> Option<&'a [T]> {
        if (self.start > self.end) || (self.start >= source.len()) || (self.end > source.len()) {
            if self.is_empty() {
                return Some(&source[0..0]);
            }
            return None;
        }

        Some(&source[self.start..self.end])
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

    pub fn set_len(&mut self, length: usize) {
        self.end = self.start + length;
    }

    pub fn with_len(&self, length: usize) -> Self {
        let mut cloned = *self;
        cloned.set_len(length);
        cloned
    }

    // Add an amount to both [`Self::start`] and [`Self::end`]
    pub fn push_by(&mut self, by: usize) {
        self.start += by;
        self.end += by;
    }

    // Subtract an amount to both [`Self::start`] and [`Self::end`]
    pub fn pull_by(&mut self, by: usize) {
        self.start -= by;
        self.end -= by;
    }

    // Add an amount to a copy of both [`Self::start`] and [`Self::end`]
    pub fn pushed_by(&self, by: usize) -> Self {
        let mut clone = *self;
        clone.start += by;
        clone.end += by;
        clone
    }

    // Subtract an amount to a copy of both [`Self::start`] and [`Self::end`]
    pub fn pulled_by(&self, by: usize) -> Option<Self> {
        if by > self.start {
            return None;
        }

        let mut clone = *self;
        clone.start -= by;
        clone.end -= by;
        Some(clone)
    }

    // Add an amount a copy of both [`Self::start`] and [`Self::end`]
    pub fn with_offset(&self, by: usize) -> Self {
        let mut clone = *self;
        clone.push_by(by);
        clone
    }
}

/// Additional functions for types that implement [`std::fmt::Debug`] and [`Display`].
impl<T: Display + std::fmt::Debug> Span<T> {
    /// Gets the content as a [`String`].
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
            Span::empty()
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
    fn from(value: Range<usize>) -> Self {
        Self::new(value.start, value.end)
    }
}

impl<T> From<Span<T>> for Range<usize> {
    fn from(value: Span<T>) -> Self {
        value.start..value.end
    }
}

impl<T> IntoIterator for Span<T> {
    type Item = usize;

    type IntoIter = Range<usize>;

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
        let token_span = Span::empty();
        let converted = token_span.to_char_span(doc.get_tokens());
        assert!(converted.is_empty());

        // Span from `Expr`.
        let token_span = SequenceExpr::default()
            .then_any_word()
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
