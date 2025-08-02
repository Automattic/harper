use crate::patterns::{WithinEditDistance, Word};
use crate::{Document, Span, Token, TokenKind};

use super::{Expr, SequenceExpr};

pub struct SimilarToPhrase {
    phrase: SequenceExpr,
    fuzzy_phrase: SequenceExpr,
}

impl SimilarToPhrase {
    /// Create an error-tolerant SequenceExpr that looks for phrases similar to (but not the same as) that contained
    /// in the provided text.
    ///
    /// This is an expensive operation, so try to only do it at startup and in tests.
    ///
    /// It will panic if your document is too complex, so only run this with curated phrases.
    pub fn from_phrase(text: &str, max_edit_dist: u8) -> Self {
        let document = Document::new_plain_english_curated(text);

        Self::from_doc(&document, max_edit_dist)
    }

    /// Create an error-tolerant SequenceExpr that looks for phrases similar to (but not the same as) that contained
    /// in the provided document.
    ///
    /// This is an expensive operation, so try to only do it at startup and in tests.
    ///
    /// It will panic if your document contains certain token types, so only run this with curated phrases.
    pub fn from_doc(document: &Document, max_edit_dist: u8) -> Self {
        let mut phrase = SequenceExpr::default();
        let mut fuzzy_phrase = SequenceExpr::default();

        for token in document.fat_tokens() {
            match token.kind {
                TokenKind::Word(_lexeme_metadata) => {
                    phrase = phrase.then(Word::from_chars(token.content.as_slice()));
                    fuzzy_phrase = fuzzy_phrase
                        .then(WithinEditDistance::new(token.content.into(), max_edit_dist));
                }
                TokenKind::Space(_) => {
                    fuzzy_phrase = fuzzy_phrase.then_whitespace();
                    phrase = phrase.then_whitespace();
                }
                TokenKind::ParagraphBreak => {
                    fuzzy_phrase = fuzzy_phrase.then_whitespace();
                    phrase = phrase.then_whitespace();
                }
                _ => panic!("Fell out of expected document formats."),
            }
        }

        Self {
            phrase,
            fuzzy_phrase,
        }
    }
}

impl Expr for SimilarToPhrase {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        if self.phrase.run(cursor, tokens, source).is_some() {
            return None;
        }
        self.fuzzy_phrase.run(cursor, tokens, source)
    }
}
