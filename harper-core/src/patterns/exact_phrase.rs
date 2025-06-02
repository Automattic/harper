use crate::{Document, Token, TokenKind};

use super::{Pattern, SequencePattern, Word};

pub struct ExactPhrase {
    inner: SequencePattern,
}

impl ExactPhrase {
    pub fn from_phrase(text: &str) -> Self {
        let document = Document::new_plain_english_curated(text);
        Self::from_document(&document)
    }

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

impl Pattern for ExactPhrase {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        self.inner.matches(tokens, source)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Document,
        patterns::{ExactPhrase, Pattern},
    };

    #[test]
    fn test_exact_does_not_mean_case_sensitive() {
        let doc_lower = Document::new_plain_english_curated("hello world");
        let doc_upper = Document::new_plain_english_curated("HELLO WORLD");
        let doc_title = Document::new_plain_english_curated("Hello World");
        let phrase = ExactPhrase::from_document(&doc_lower);
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
