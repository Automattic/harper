//! Adds support for parsing various programming and markup languages through a unified trait: [`Parser`].

mod collapse_identifiers;
mod isolate_english;
mod markdown;
mod mask;
mod org_mode;
mod plain_english;

use blanket::blanket;
pub use collapse_identifiers::CollapseIdentifiers;
pub use isolate_english::IsolateEnglish;
pub use markdown::{Markdown, MarkdownOptions};
pub use mask::Mask;
pub use org_mode::OrgMode;
pub use plain_english::PlainEnglish;

use crate::{LSend, Token, TokenStringExt};

#[cfg_attr(feature = "concurrent", blanket(derive(Box, Arc)))]
#[cfg_attr(not(feature = "concurrent"), blanket(derive(Box, Rc)))]
pub trait Parser: LSend {
    fn parse(&self, source: &[char]) -> Vec<Token>;
}

pub trait StrParser {
    fn parse_str(&self, source: impl AsRef<str>) -> Vec<Token>;
}

impl<T> StrParser for T
where
    T: Parser,
{
    fn parse_str(&self, source: impl AsRef<str>) -> Vec<Token> {
        let source: Vec<_> = source.as_ref().chars().collect();
        self.parse(&source)
    }
}

#[cfg(test)]
mod tests {
    use super::{Markdown, OrgMode, Parser, PlainEnglish};
    use crate::Punctuation;
    use crate::TokenKind::{self, *};

    fn assert_tokens_eq(test_str: impl AsRef<str>, expected: &[TokenKind], parser: &impl Parser) {
        let chars: Vec<_> = test_str.as_ref().chars().collect();
        let tokens = parser.parse(&chars);
        let kinds: Vec<_> = tokens.into_iter().map(|v| v.kind).collect();

        assert_eq!(&kinds, expected)
    }

    fn assert_tokens_eq_plain(test_str: impl AsRef<str>, expected: &[TokenKind]) {
        assert_tokens_eq(test_str, expected, &PlainEnglish);
    }

    fn assert_tokens_eq_md(test_str: impl AsRef<str>, expected: &[TokenKind]) {
        assert_tokens_eq(test_str, expected, &Markdown::default())
    }

    fn assert_tokens_eq_org(test_str: impl AsRef<str>, expected: &[TokenKind]) {
        assert_tokens_eq(test_str, expected, &OrgMode)
    }

    #[test]
    fn single_letter() {
        assert_tokens_eq_plain("a", &[TokenKind::blank_word()])
    }

    #[test]
    fn sentence() {
        assert_tokens_eq_plain(
            "hello world, my friend",
            &[
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
                Punctuation(Punctuation::Comma),
                Space(1),
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
            ],
        )
    }

    #[test]
    fn sentence_md() {
        assert_tokens_eq_md(
            "__hello__ world, [my]() friend",
            &[
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
                Punctuation(Punctuation::Comma),
                Space(1),
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
            ],
        );
    }

    #[test]
    fn inserts_newlines() {
        assert_tokens_eq_md(
            "__hello__ world,\n\n[my]() friend",
            &[
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
                Punctuation(Punctuation::Comma),
                ParagraphBreak,
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
            ],
        );
    }

    /// Make sure that the English parser correctly identifies non-English
    /// characters as part of the same word.
    #[test]
    fn parses_non_english() {
        assert_tokens_eq_plain("Løvetann", &[TokenKind::blank_word()]);
        assert_tokens_eq_plain("Naïve", &[TokenKind::blank_word()]);
    }

    #[test]
    fn org_mode_basic() {
        assert_tokens_eq_org(
            "hello world",
            &[TokenKind::blank_word(), Space(1), TokenKind::blank_word()],
        );
    }
}
