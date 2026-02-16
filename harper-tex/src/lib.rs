mod masker;

use harper_core::Token;
use harper_core::parsers::{Mask, Parser, PlainEnglish};

use self::masker::Masker;

/// A parser for Harper that wraps the native `PlainEnglish` parser, allowing one use Harper on
/// documents written in TeX, LaTeX, or any other TeX derivative.
///
/// This parser is crude, and could definitely use work if all features of Harper wish to be
/// supported for the language.
pub struct TeX {
    inner: Mask<Masker, PlainEnglish>,
}

impl Default for TeX {
    fn default() -> Self {
        Self {
            inner: Mask::new(Default::default(), PlainEnglish),
        }
    }
}

impl Parser for TeX {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}

#[cfg(test)]
mod tests {
    use harper_core::TokenKind;
    use harper_core::parsers::StrParser;

    use crate::TeX;

    #[test]
    fn ignores_comment_characters() {
        let source = r"%!!%";

        let toks = TeX::default().parse_str(source);
        let tok_kinds: Vec<_> = toks.into_iter().map(|t| t.kind).collect();

        assert_eq!(
            tok_kinds,
            vec![
                TokenKind::Punctuation(harper_core::Punctuation::Bang),
                TokenKind::Punctuation(harper_core::Punctuation::Bang),
            ]
        )
    }

    #[test]
    fn passes_comment_characters_preceded_by_backslash() {
        let source = r"\%!!";

        let toks = TeX::default().parse_str(source);
        let tok_kinds: Vec<_> = toks.into_iter().map(|t| t.kind).collect();

        assert_eq!(
            tok_kinds,
            vec![
                TokenKind::Punctuation(harper_core::Punctuation::Bang),
                TokenKind::Punctuation(harper_core::Punctuation::Bang)
            ]
        )
    }
}
