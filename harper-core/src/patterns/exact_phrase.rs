use std::num::NonZeroUsize;

use crate::{Document, Token, TokenKind};

use super::{AnyCapitalization, Pattern, SequencePattern};

pub struct ExactPhrase {
    inner: SequencePattern,
}

impl ExactPhrase {
    pub fn from_phrase(text: &str) -> Self {
        let document = Document::new_markdown_default_curated(text);
        Self::from_document(&document)
    }

    pub fn from_document(doc: &Document) -> Self {
        let mut phrase = SequencePattern::default();

        for token in doc.fat_tokens() {
            match token.kind {
                TokenKind::Word(_word_metadata) => {
                    phrase = phrase.then(AnyCapitalization::new(token.content.as_slice().into()));
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

impl Pattern for ExactPhrase {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<NonZeroUsize> {
        self.inner.matches(tokens, source)
    }
}
