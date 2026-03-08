use crate::{Span, Token};
use itertools::Itertools;
use paste::paste;

macro_rules! create_fns_for {
    ($thing:ident) => {
        paste! {
            fn [< first_ $thing >](&self) -> Option<&Token> {
                self.tokens().iter().find(|v| v.kind.[<is_ $thing>]())
            }

            fn [< last_ $thing >](&self) -> Option<&Token> {
                self.tokens().iter().rev().find(|v| v.kind.[<is_ $thing>]())
            }

            fn [< last_ $thing _index >](&self) -> Option<usize> {
                let tokens = self.tokens();

                tokens.iter().rev().position(|v| v.kind.[<is_ $thing>]()).map(|i| tokens.len() - i - 1)
            }

            fn [<iter_ $thing _indices>](&self) -> impl DoubleEndedIterator<Item = usize> + '_ {
                self.tokens().iter()
                    .enumerate()
                    .filter(|(_, t)| t.kind.[<is_ $thing>]())
                    .map(|(i, _)| i)
            }

            fn [<iter_ $thing s>](&self) -> impl Iterator<Item = &Token> + '_ {
                let tokens = self.tokens();

                tokens.[<iter_ $thing _indices>]().map(|i| &tokens[i])
            }
        }
    };
}

mod private {
    use crate::{Document, Token};

    pub trait Sealed {}

    impl Sealed for [Token] {}

    impl Sealed for Document {}
}

/// Extension methods for [`Token`] sequences that make them easier to wrangle and query.
pub trait TokenStringExt: private::Sealed {
    // Used by the default implementations.
    fn tokens(&self) -> &[Token];

    // Used by the default implementations.
    fn tokens_mut(&mut self) -> &mut [Token];

    create_fns_for!(adjective);
    create_fns_for!(apostrophe);
    create_fns_for!(at);
    create_fns_for!(comma);
    create_fns_for!(conjunction);
    create_fns_for!(chunk_terminator);
    create_fns_for!(currency);
    create_fns_for!(ellipsis);
    create_fns_for!(hostname);
    create_fns_for!(likely_homograph);
    create_fns_for!(number);
    create_fns_for!(noun);
    create_fns_for!(paragraph_break);
    create_fns_for!(pipe);
    create_fns_for!(preposition);
    create_fns_for!(punctuation);
    create_fns_for!(quote);
    create_fns_for!(sentence_terminator);
    create_fns_for!(space);
    create_fns_for!(unlintable);
    create_fns_for!(verb);
    create_fns_for!(word);
    create_fns_for!(word_like);
    create_fns_for!(heading_start);

    fn first_sentence_word(&self) -> Option<&Token> {
        let tokens = self.tokens();

        let (w_idx, word) = tokens.iter().find_position(|v| v.kind.is_word())?;

        let Some(u_idx) = tokens.iter().position(|v| v.kind.is_unlintable()) else {
            return Some(word);
        };

        if w_idx < u_idx { Some(word) } else { None }
    }

    fn first_non_whitespace(&self) -> Option<&Token> {
        self.tokens().iter().find(|t| !t.kind.is_whitespace())
    }

    /// Grab the span that represents the beginning of the first element and the
    /// end of the last element.
    fn span(&self) -> Option<Span<char>> {
        let min_max = self
            .tokens()
            .iter()
            .flat_map(|v| [v.span.start, v.span.end].into_iter())
            .minmax();

        match min_max {
            itertools::MinMaxResult::NoElements => None,
            itertools::MinMaxResult::OneElement(min) => Some(Span::new(min, min)),
            itertools::MinMaxResult::MinMax(min, max) => Some(Span::new(min, max)),
        }
    }

    /// Get a reference to a token by index, with negative numbers counting from the end.
    ///
    /// # Examples
    /// ```
    /// # use harper_core::{Token, TokenStringExt, parsers::{Parser, PlainEnglish}};
    /// # fn main() {
    /// let source = "The cat sat on the mat.".chars().collect::<Vec<_>>();
    /// let tokens = PlainEnglish.parse(&source);
    /// assert_eq!(tokens.get_rel(0).unwrap().span.get_content_string(&source), "The");
    /// assert_eq!(tokens.get_rel(1).unwrap().kind.is_whitespace(), true);
    /// assert_eq!(tokens.get_rel(-1).unwrap().kind.is_punctuation(), true);
    /// assert_eq!(tokens.get_rel(-2).unwrap().span.get_content_string(&source), "mat");
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// * `Some(&Token)` - If the index is in bounds
    /// * `None` - If the index is out of bounds
    fn get_rel(&self, index: isize) -> Option<&Token>
    where
        Self: AsRef<[Token]>,
    {
        let slice = self.as_ref();
        let len = slice.len() as isize;

        if index >= len || -index > len {
            return None;
        }

        let idx = if index >= 0 { index } else { len + index } as usize;

        slice.get(idx)
    }

    /// Get a slice of tokens using relative indices.
    ///
    /// # Examples
    /// ```
    /// # use harper_core::{Token, TokenStringExt, parsers::{Parser, PlainEnglish}};
    /// # fn main() {
    /// let source = "The cat sat on the mat.".chars().collect::<Vec<_>>();
    /// let tokens = PlainEnglish.parse(&source);
    /// assert_eq!(tokens.get_rel_slice(0, 2).unwrap().span().unwrap().get_content_string(&source), "The cat");
    /// assert_eq!(tokens.get_rel_slice(-3, -1).unwrap().span().unwrap().get_content_string(&source), " mat.");
    /// # }
    /// ```
    fn get_rel_slice(&self, rel_start: isize, inclusive_end: isize) -> Option<&[Token]>
    where
        Self: AsRef<[Token]>,
    {
        let slice = self.as_ref();
        let len = slice.len() as isize;

        // Convert relative indices to absolute indices
        let start_idx = if rel_start >= 0 {
            rel_start
        } else {
            len + rel_start
        } as usize;

        let end_idx_plus_one = if inclusive_end >= 0 {
            inclusive_end + 1 // +1 to make end exclusive
        } else {
            len + inclusive_end + 1
        } as usize;

        // Check bounds
        if start_idx >= slice.len()
            || end_idx_plus_one > slice.len()
            || start_idx >= end_idx_plus_one
        {
            return None;
        }

        Some(&slice[start_idx..end_idx_plus_one])
    }

    fn iter_linking_verb_indices(&self) -> impl Iterator<Item = usize> + '_ {
        let tokens = self.tokens();

        tokens.iter_word_indices().filter(|idx| {
            let word = &tokens[*idx];
            let Some(Some(meta)) = word.kind.as_word() else {
                return false;
            };

            meta.is_linking_verb()
        })
    }

    fn iter_linking_verbs(&self) -> impl Iterator<Item = &Token> + '_ {
        let tokens = self.tokens();

        tokens.iter_linking_verb_indices().map(|idx| &tokens[idx])
    }

    /// Iterate over chunks.
    ///
    /// For example, the following sentence contains two chunks separated by a
    /// comma:
    ///
    /// ```text
    /// Here is an example, it is short.
    /// ```
    fn iter_chunks(&self) -> impl Iterator<Item = &'_ [Token]> + '_ {
        self.tokens()
            .split_inclusive(|tok| tok.kind.is_chunk_terminator())
    }

    /// Get an iterator over token slices that represent the individual
    /// paragraphs in a document.
    fn iter_paragraphs(&self) -> impl Iterator<Item = &'_ [Token]> + '_ {
        self.tokens()
            .split_inclusive(|tok| tok.kind.is_paragraph_break())
    }

    /// Get an iterator over token slices that represent headings.
    ///
    /// A heading begins with a [`TokenKind::HeadingStart`](crate::TokenKind::HeadingStart) token and ends with
    /// the next [`TokenKind::ParagraphBreak`](crate::TokenKind::ParagraphBreak).
    fn iter_headings(&self) -> impl Iterator<Item = &'_ [Token]> + '_ {
        let tokens = self.tokens();

        tokens.iter_heading_start_indices().map(|start| {
            let end = tokens[start..]
                .iter()
                .position(|t| t.kind.is_paragraph_break())
                .unwrap_or(tokens[start..].len() - 1);

            &tokens[start..=start + end]
        })
    }

    /// Get an iterator over token slices that represent the individual
    /// sentences in a document.
    fn iter_sentences(&self) -> impl Iterator<Item = &'_ [Token]> + '_ {
        self.tokens()
            .split_inclusive(|tok| tok.kind.is_sentence_terminator())
    }

    /// Get an iterator over mutable token slices that represent the individual
    /// sentences in a document.
    fn iter_sentences_mut(&mut self) -> impl Iterator<Item = &'_ mut [Token]> + '_ {
        struct SentIter<'a> {
            rem: &'a mut [Token],
        }

        impl<'a> Iterator for SentIter<'a> {
            type Item = &'a mut [Token];

            fn next(&mut self) -> Option<Self::Item> {
                if self.rem.is_empty() {
                    return None;
                }
                let split = self
                    .rem
                    .iter()
                    .position(|t| t.kind.is_sentence_terminator())
                    .map(|i| i + 1)
                    .unwrap_or(self.rem.len());
                let tmp = core::mem::take(&mut self.rem);
                let (sent, rest) = tmp.split_at_mut(split);
                self.rem = rest;
                Some(sent)
            }
        }

        let tokens = self.tokens_mut();

        SentIter { rem: tokens }
    }
}

impl TokenStringExt for [Token] {
    fn tokens(&self) -> &[Token] {
        self
    }

    fn tokens_mut(&mut self) -> &mut [Token] {
        self
    }
}
