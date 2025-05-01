use std::fmt::Display;

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
    pub fn replace_with_match_case_str(value: &'static str, template: &[char]) -> Self {
        Self::replace_with_match_case(value.chars().collect(), template)
    }

    /// Construct an instance of [`Self::ReplaceWith`], but make the content match the case of the
    /// provided template.
    ///
    /// For example, if we want to replace "You're" with "You are", we can provide "you are" and
    /// "You're".
    pub fn replace_with_match_case(mut value: Vec<char>, template: &[char]) -> Self {
        // If the value is longer than the template, use this.
        let template_term = [template.last().copied().unwrap_or('l')]
            .into_iter()
            .cycle();

        for (v, t) in value.iter_mut().filter(|v| v.is_alphabetic()).zip(
            template
                .iter()
                .filter(|v| v.is_alphabetic())
                .copied()
                .chain(template_term),
        ) {
            if v.is_ascii_uppercase() != t.is_ascii_uppercase() {
                if t.is_uppercase() {
                    *v = v.to_ascii_uppercase();
                } else {
                    *v = v.to_ascii_lowercase();
                }
            }
        }

        Self::ReplaceWith(value)
    }

    /// Apply a suggestion to a given text.
    pub fn apply(&self, span: Span, source: &mut Vec<char>) {
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
