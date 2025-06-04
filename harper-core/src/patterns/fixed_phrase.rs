use crate::{Document, Token, TokenKind};

use super::{Pattern, SequencePattern, Word};

/// Matches a fixed sequence of tokens as they appear in the input.
/// Case-insensitive for words but maintains exact matching for other token types.
///
/// # Example
///
/// ```rust
/// use harper_core::patterns::{FixedPhrase, Pattern};
/// use harper_core::Document;
///
/// let doc = Document::new_plain_english_curated("Hello, world!");
/// let phrase = FixedPhrase::from_phrase("Hello, world!");
/// assert!(phrase.matches(doc.get_tokens(), doc.get_source()).is_some());
/// ```
pub struct FixedPhrase {
    inner: SequencePattern,
}

impl FixedPhrase {
    /// Creates a [FixedPhrase] from a plain text string.
    /// Uses plain English tokenization rules.
    pub fn from_phrase(text: &str) -> Self {
        let document = Document::new_plain_english_curated(text);
        Self::from_document(&document)
    }

    /// Creates a [FixedPhrase] from a pre-tokenized document.
    /// Allows custom tokenization by creating a `Document` first.
    pub fn from_document(doc: &Document) -> Self {
        let mut phrase = SequencePattern::default();

        for token in doc.fat_tokens() {
            match token.kind {
                TokenKind::Word(_word_metadata) => {
                    phrase = phrase.then(Word::from_chars(token.content.as_slice()));
                }
                TokenKind::Space(_) => {
                    phrase = phrase.then_whitespace();
                }
                TokenKind::Punctuation(p) => {
                    phrase = phrase.then(move |t: &Token, _source: &[char]| {
                        t.kind.as_punctuation().cloned() == Some(p)
                    })
                }
                TokenKind::ParagraphBreak => {
                    phrase = phrase.then_whitespace();
                }
                TokenKind::Number(n) => {
                    phrase = phrase
                        .then(move |tok: &Token, _source: &[char]| tok.kind == TokenKind::Number(n))
                }
                _ => panic!("Fell out of expected document formats."),
            }
        }

        Self { inner: phrase }
    }
}

impl Pattern for FixedPhrase {
    /// Matches this phrase against the start of the token slice.
    /// Returns `Some(len)` on match (number of tokens consumed), `None` otherwise.
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        self.inner.matches(tokens, source)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Document,
        patterns::{FixedPhrase, Pattern},
    };

    #[test]
    fn test_not_case_sensitive() {
        let doc_lower = Document::new_plain_english_curated("hello world");
        let doc_upper = Document::new_plain_english_curated("HELLO WORLD");
        let doc_title = Document::new_plain_english_curated("Hello World");
        let phrase = FixedPhrase::from_document(&doc_lower);
        assert_eq!(
            phrase.matches(doc_lower.get_tokens(), doc_title.get_source()),
            Some(3)
        );
        assert_eq!(
            phrase.matches(doc_lower.get_tokens(), doc_upper.get_source()),
            Some(3)
        );
        assert_eq!(
            phrase.matches(doc_title.get_tokens(), doc_lower.get_source()),
            Some(3)
        );
        assert_eq!(
            phrase.matches(doc_title.get_tokens(), doc_upper.get_source()),
            Some(3)
        );
        assert_eq!(
            phrase.matches(doc_upper.get_tokens(), doc_lower.get_source()),
            Some(3)
        );
        assert_eq!(
            phrase.matches(doc_upper.get_tokens(), doc_title.get_source()),
            Some(3)
        );
    }
}
