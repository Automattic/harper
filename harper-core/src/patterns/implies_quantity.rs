use crate::{Token, TokenKind};

use super::SingleTokenPattern;

/// This struct does two things.
///
/// First, it acts as a pattern that looks for phrases that describe a quantity of a noun
/// that may or may not succeed it.
///
/// Second, it determines the implied plurality of that quantity.implies
pub struct ImpliesQuantity;

impl ImpliesQuantity {
    pub fn implies_plurality(token: &Token, source: &[char]) -> Option<bool> {
        match &token.kind {
            TokenKind::Word(Some(lexeme_metadata)) => {
                if lexeme_metadata.is_determiner() {
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

impl SingleTokenPattern for ImpliesQuantity {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        Self::implies_plurality(token, source).is_some()
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
