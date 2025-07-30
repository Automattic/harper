use std::num::NonZeroUsize;

use crate::{Token, TokenKind};

use super::Pattern;

/// This struct does two things.
///
/// First, it acts as a pattern that looks for phrases that describe a quantity of a noun
/// that may or may not succeed it.
///
/// Second, it determines the implied plurality of that quantity.implies
pub struct ImpliesQuantity;

impl ImpliesQuantity {
    pub fn implies_plurality(tokens: &[Token], source: &[char]) -> Option<bool> {
        let token = tokens.first()?;

        match &token.kind {
            TokenKind::Word(Some(word_metadata)) => {
                if word_metadata.determiner {
                    return Some(false);
                }

                let source = token.span.get_content(source);

                match source {
                    ['a'] => Some(false),
                    ['a', 'n'] => Some(false),
                    ['m', 'a', 'n', 'y'] => Some(true),
                    _ => None,
                }
            }
            TokenKind::Number(number) => Some((number.value.abs() - 1.).abs() > f64::EPSILON),
            _ => None,
        }
    }
}

impl Pattern for ImpliesQuantity {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        NonZeroUsize::new(if Self::implies_plurality(tokens, source).is_some() {
            1
        } else {
            0
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Document, Span,
        patterns::{DocPattern, ImpliesQuantity},
    };

    #[test]
    fn number_implies() {
        let doc = Document::new_plain_english_curated("There are 60 minutes in an hour.");

        assert_eq!(
            ImpliesQuantity.find_all_matches_in_doc(&doc),
            vec![Span::new(4, 5), Span::new(10, 11)]
        )
    }
}
