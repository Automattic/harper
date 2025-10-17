use std::{borrow::Borrow, fmt::Display};

use is_macro::Is;
use serde::{Deserialize, Serialize};

use crate::Span;

/// A suggested edit that could resolve a [`Lint`](super::Lint).
#[derive(Debug, Clone, Serialize, Deserialize, Is, PartialEq, Eq, Hash)]
pub enum Suggestion {
    /// Replace the offending text with a specific character sequence.
    ReplaceWith(Vec<char>),
    /// Insert the provided characters _after_ the offending text.
    InsertAfter(Vec<char>),
    /// Remove the offending text.
    Remove,
}

impl Suggestion {
    /// Variant of [`Self::replace_with_match_case`] that accepts a static string.
    pub fn replace_with_match_case_str(
        value: &str,
        template: impl IntoIterator<Item = impl Borrow<char>>,
    ) -> Self {
        Self::replace_with_match_case(value.chars().collect(), template)
    }

    /// Construct an instance of [`Self::ReplaceWith`], but make the content match the case of the
    /// provided template.
    ///
    /// For example, if we want to replace "You're" with "You are", we can provide "you are" and
    /// "You're".
    pub fn replace_with_match_case(
        mut value: Vec<char>,
        template: impl IntoIterator<Item = impl Borrow<char>>,
    ) -> Self {
        let mut template = template.into_iter();
        // Repeat the last character if we still need characters past the end of `template`.
        let template = itertools::iterate(
            template.next().map(|first_char| *first_char.borrow()),
            |prev_c| {
                if let Some(c) = template.next() {
                    Some(*c.borrow())
                } else {
                    *prev_c
                }
            },
        );

        for (v, t) in value
            .iter_mut()
            .filter(|v| v.is_alphabetic())
            .zip(template.filter(|v| v.is_none_or(|v| v.is_alphabetic())))
        {
            if t.is_some_and(|t| t.is_uppercase()) {
                *v = v.to_ascii_uppercase();
            } else {
                *v = v.to_ascii_lowercase();
            }
        }

        Self::ReplaceWith(value)
    }

    /// Apply a suggestion to a given text.
    pub fn apply(&self, span: Span<char>, source: &mut Vec<char>) {
        match self {
            Self::ReplaceWith(chars) => {
                // Avoid allocation if possible
                if chars.len() == span.len() {
                    for (index, c) in chars.iter().enumerate() {
                        source[index + span.start] = *c
                    }
                } else {
                    let popped = source.split_off(span.start);

                    source.extend(chars);
                    source.extend(popped.into_iter().skip(span.len()));
                }
            }
            Self::Remove => {
                for i in span.end..source.len() {
                    source[i - span.len()] = source[i];
                }

                source.truncate(source.len() - span.len());
            }
            Self::InsertAfter(chars) => {
                let popped = source.split_off(span.end);
                source.extend(chars);
                source.extend(popped);
            }
        }
    }
}

impl Display for Suggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Suggestion::ReplaceWith(with) => {
                write!(f, "Replace with: “{}”", with.iter().collect::<String>())
            }
            Suggestion::InsertAfter(with) => {
                write!(f, "Insert “{}”", with.iter().collect::<String>())
            }
            Suggestion::Remove => write!(f, "Remove error"),
        }
    }
}

pub trait SuggestionCollectionExt {
    fn to_replace_suggestions(
        self,
        case_template: impl IntoIterator<Item = impl Borrow<char>> + Clone,
    ) -> impl Iterator<Item = Suggestion>;
}

impl<I, T> SuggestionCollectionExt for I
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    fn to_replace_suggestions(
        self,
        case_template: impl IntoIterator<Item = impl Borrow<char>> + Clone,
    ) -> impl Iterator<Item = Suggestion> {
        self.into_iter().map(move |s| {
            Suggestion::replace_with_match_case_str(s.as_ref(), case_template.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Span;

    use super::Suggestion;

    #[test]
    fn insert_comma_after() {
        let source = "This is a test";
        let mut source_chars = source.chars().collect();
        let sug = Suggestion::InsertAfter(vec![',']);
        sug.apply(Span::new(0, 4), &mut source_chars);

        assert_eq!(source_chars, "This, is a test".chars().collect::<Vec<_>>());
    }

    #[test]
    fn suggestion_your_match_case() {
        let template: Vec<_> = "You're".chars().collect();
        let value: Vec<_> = "you are".chars().collect();

        let correct = "You are".chars().collect();

        assert_eq!(
            Suggestion::replace_with_match_case(value, &template),
            Suggestion::ReplaceWith(correct)
        )
    }

    #[test]
    fn issue_1065() {
        let template: Vec<_> = "Stack Overflow".chars().collect();
        let value: Vec<_> = "stackoverflow".chars().collect();

        let correct = "StackOverflow".chars().collect();

        assert_eq!(
            Suggestion::replace_with_match_case(value, &template),
            Suggestion::ReplaceWith(correct)
        )
    }
}
