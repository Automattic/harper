use super::{Atom, Concat, Quantifier, RegexNode};

/// Helper functions for flexible whitespace matching.
///
/// These are essential for handling punctuation spacing errors like:
/// - "you,me,and him" (missing spaces after commas)
/// - "him ,her ,and it" (spaces before commas)
/// - "word,word" (no space after punctuation)
pub struct Spacing;

impl Spacing {
    /// Match zero or more whitespace tokens.
    ///
    /// This is useful for optional spacing between tokens.
    pub fn optional_whitespace() -> Box<dyn RegexNode> {
        Box::new(Quantifier::zero_or_more(Box::new(Atom::whitespace())))
    }

    /// Match one or more whitespace tokens.
    ///
    /// This is useful for required spacing between tokens.
    pub fn required_whitespace() -> Box<dyn RegexNode> {
        Box::new(Quantifier::one_or_more(Box::new(Atom::whitespace())))
    }

    /// Match a pattern with optional whitespace before and after.
    ///
    /// Useful for matching punctuation that may or may not have spacing.
    pub fn with_optional_spacing(pattern: Box<dyn RegexNode>) -> Box<dyn RegexNode> {
        Box::new(Concat::new(vec![
            Self::optional_whitespace(),
            pattern,
            Self::optional_whitespace(),
        ]))
    }

    /// Match two patterns with optional whitespace between them.
    ///
    /// This handles cases like "word,word" vs "word, word" vs "word , word".
    pub fn spaced(left: Box<dyn RegexNode>, right: Box<dyn RegexNode>) -> Box<dyn RegexNode> {
        Box::new(Concat::new(vec![left, Self::optional_whitespace(), right]))
    }

    /// Match a pattern with flexible spacing around punctuation.
    ///
    /// This handles all variations of punctuation spacing:
    /// - "word,word" (no spaces)
    /// - "word, word" (space after)
    /// - "word ,word" (space before)
    /// - "word , word" (spaces both sides)
    pub fn flexible_punctuation(punctuation: Box<dyn RegexNode>) -> Box<dyn RegexNode> {
        Self::with_optional_spacing(punctuation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::rig::Atom;

    #[test]
    fn test_optional_whitespace() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let ws = Spacing::optional_whitespace();
        let result = ws.exec(tokens, source, 0);

        // Should match zero whitespace at start
        assert!(result.is_some());
        assert_eq!(result.unwrap().tokens_consumed, 0);
    }

    #[test]
    fn test_required_whitespace() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let ws = Spacing::required_whitespace();
        let result = ws.exec(tokens, source, 1); // At the space token

        assert!(result.is_some());
        assert_eq!(result.unwrap().tokens_consumed, 1);
    }

    #[test]
    fn test_with_optional_spacing() {
        let doc = Document::new_plain_english_curated("hello,world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let pattern = Spacing::with_optional_spacing(Box::new(Atom::word("hello")));
        let result = pattern.exec(tokens, source, 0);

        assert!(result.is_some());
    }

    #[test]
    fn test_spaced() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let pattern = Spacing::spaced(Box::new(Atom::word("hello")), Box::new(Atom::word("world")));
        let result = pattern.exec(tokens, source, 0);

        assert!(result.is_some());
    }
}
