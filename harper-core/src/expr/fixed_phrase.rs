use crate::patterns::Word;
use crate::{Document, Span, Token, TokenKind};

use super::{Expr, SequenceExpr};

/// Matches a fixed sequence of tokens as they appear in the input.
/// Case-insensitive for words but maintains exact matching for other token types.
///
/// # Example
///
/// ```rust
/// use harper_core::expr::{FixedPhrase, Expr};
/// use harper_core::Document;
///
/// let doc = Document::new_plain_english_curated("Hello, world!");
/// let phrase = FixedPhrase::from_phrase("Hello, world!");
/// assert!(phrase.run(0, doc.get_tokens(), doc.get_source()).is_some());
/// ```
pub struct FixedPhrase {
    inner: SequenceExpr,
}

impl FixedPhrase {
    /// Creates a [`FixedPhrase`] from a plaintext string.
    /// Uses plain English tokenization rules.
    pub fn from_phrase(text: &str) -> Self {
        let document = Document::new_plain_english_curated(text);
        Self::from_document(&document)
    }

    /// Creates a [`FixedPhrase`] from a pre-tokenized document.
    /// Allows custom tokenization by creating a `Document` first.
    pub fn from_document(doc: &Document) -> Self {
        let mut phrase = SequenceExpr::default();

        for token in doc.fat_tokens() {
            match token.kind {
                TokenKind::Word(_lexeme_metadata) => {
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

impl Expr for FixedPhrase {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        self.inner.run(cursor, tokens, source)
    }
}

#[cfg(test)]
mod tests {
    use super::FixedPhrase;
    use crate::expr::Expr;
    use crate::{Document, Span};

    #[test]
    fn test_not_case_sensitive() {
        let doc_lower = Document::new_plain_english_curated("hello world");
        let doc_upper = Document::new_plain_english_curated("HELLO WORLD");
        let doc_title = Document::new_plain_english_curated("Hello World");
        let phrase = FixedPhrase::from_document(&doc_lower);
        assert_eq!(
            phrase.run(0, doc_lower.get_tokens(), doc_title.get_source()),
            Some(Span::new(0, 3))
        );
        assert_eq!(
            phrase.run(0, doc_lower.get_tokens(), doc_upper.get_source()),
            Some(Span::new(0, 3))
        );
        assert_eq!(
            phrase.run(0, doc_title.get_tokens(), doc_lower.get_source()),
            Some(Span::new(0, 3))
        );
        assert_eq!(
            phrase.run(0, doc_title.get_tokens(), doc_upper.get_source()),
            Some(Span::new(0, 3))
        );
        assert_eq!(
            phrase.run(0, doc_upper.get_tokens(), doc_lower.get_source()),
            Some(Span::new(0, 3))
        );
        assert_eq!(
            phrase.run(0, doc_upper.get_tokens(), doc_title.get_source()),
            Some(Span::new(0, 3))
        );
    }
}
