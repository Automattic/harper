use serde::{Deserialize, Serialize};

use crate::{FatToken, Span, TokenKind};

/// Represents a semantic, parsed component of a [`Document`](crate::Document).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Token {
    /// The characters the token represents.
    pub span: Span<char>,
    /// The parsed value.
    pub kind: TokenKind,
}

impl Token {
    pub fn new(span: Span<char>, kind: TokenKind) -> Self {
        Self { span, kind }
    }

    /// Get the token's content as a slice of characters.
    pub fn get_ch<'a>(&self, source: &'a [char]) -> &'a [char] {
        self.span.get_content(source)
    }

    /// Get the token's content as a string.
    pub fn get_str(&self, source: &[char]) -> String {
        self.span.get_content_string(source)
    }

    /// Convert to an allocated [`FatToken`].
    pub fn to_fat(&self, source: &[char]) -> FatToken {
        let content = self.get_ch(source).to_vec();

        FatToken {
            content,
            kind: self.kind.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Span, Token, TokenKind, TokenStringExt,
        parsers::{Parser, PlainEnglish},
    };

    #[test]
    fn unlintable_token_is_detected() {
        let token = Token::new(Span::new(0, 5), TokenKind::Unlintable);
        assert!(token.kind_is_unlintable());
    }

    #[test]
    fn word_token_is_not_unlintable() {
        let token = Token::new(
            Span::new(0, 5),
            TokenKind::Word(crate::WordMetadata::default()),
        );
        assert!(!token.kind_is_unlintable());
    }

    #[test]
    fn newline_token_is_not_unlintable() {
        let token = Token::new(Span::new(0, 1), TokenKind::Newline(1));
        assert!(!token.kind_is_unlintable());
    }

    #[test]
    fn parses_sentences_correctly() {
        let text = "There were three little pigs. They built three little homes.";
        let chars: Vec<char> = text.chars().collect();
        let toks = PlainEnglish.parse(&chars);

        let mut sentence_strs = vec![];

        for sentence in toks.iter_sentences() {
            if let Some(span) = sentence.span() {
                sentence_strs.push(span.get_content_string(&chars));
            }
        }

        assert_eq!(
            sentence_strs,
            vec![
                "There were three little pigs.",
                " They built three little homes."
            ]
        )
    }
}
