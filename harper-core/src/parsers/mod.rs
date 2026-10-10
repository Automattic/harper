//! Adds support for parsing various programming and markup languages through a unified trait: [`Parser`].

mod collapse_identifiers;
mod isolate_english;
mod markdown;
mod mask;
mod oops_all_headings;
mod org_mode;
mod plain_english;

use blanket::blanket;
pub use collapse_identifiers::CollapseIdentifiers;
pub use isolate_english::IsolateEnglish;
pub use markdown::{Markdown, MarkdownOptions};
pub use mask::Mask;
pub use oops_all_headings::OopsAllHeadings;
pub use org_mode::OrgMode;
pub use plain_english::PlainEnglish;

use crate::{LSend, Token, TokenStringExt};

#[cfg_attr(feature = "concurrent", blanket(derive(Ref, Box, Arc)))]
#[cfg_attr(not(feature = "concurrent"), blanket(derive(Ref, Box, Rc)))]
pub trait Parser: LSend {
    fn parse(&self, source: &[char]) -> Vec<Token>;

    /// Whether the text this parser produces is English prose.
    ///
    /// [`Document`](crate::Document) runs the Brill part-of-speech tagger and
    /// the neural noun-phrase chunker over every sentence it builds. Both are
    /// trained on English alone, so on other languages they burn most of the
    /// parse budget to produce tags that are absent or wrong. A parser that
    /// answers `false` opts out of both.
    ///
    /// The default is `true`, so English and any parser written before this
    /// existed keep their behaviour exactly.
    fn is_english(&self) -> bool {
        true
    }
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
    use super::{Markdown, MarkdownOptions, OrgMode, Parser, PlainEnglish};
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
        assert_tokens_eq(test_str, expected, &OrgMode::default())
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

    /// Stands in for a language module's plain parser.
    struct NotEnglish;

    impl Parser for NotEnglish {
        fn parse(&self, source: &[char]) -> Vec<crate::Token> {
            PlainEnglish.parse(source)
        }

        fn is_english(&self) -> bool {
            false
        }
    }

    #[test]
    fn english_is_the_default_answer() {
        assert!(PlainEnglish.is_english());
        assert!(Markdown::default().is_english());
        assert!(OrgMode::default().is_english());
    }

    #[test]
    fn markdown_and_org_take_the_language_of_their_inline_parser() {
        let markdown =
            Markdown::with_inline_parser(MarkdownOptions::default(), |s| NotEnglish.parse(s));
        assert!(markdown.is_english());
        assert!(!markdown.non_english().is_english());

        let org = OrgMode::with_inline_parser(|s| NotEnglish.parse(s));
        assert!(org.is_english());
        assert!(!org.non_english().is_english());
    }

    /// Wrapping a parser changes what gets parsed, never which language it is
    /// in, so the answer has to travel through the wrapper.
    #[test]
    fn wrapping_parsers_forward_the_language() {
        use super::{Mask, OopsAllHeadings};
        use crate::mask::RegexMasker;

        assert!(!OopsAllHeadings::new(NotEnglish).is_english());
        assert!(OopsAllHeadings::new(PlainEnglish).is_english());

        let masker = || RegexMasker::new(".*", true).unwrap();
        assert!(!Mask::new(masker(), NotEnglish).is_english());
        assert!(Mask::new(masker(), PlainEnglish).is_english());
    }
}
