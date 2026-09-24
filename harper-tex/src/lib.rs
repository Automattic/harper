mod masker;

use harper_core::parsers::{Mask, Parser, PlainEnglish};
use harper_core::{Punctuation, Span, Token, TokenKind};

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
        let tokens = insert_inline_math_tokens(self.inner.parse(source), source);
        collapse_tex_dashes(tokens)
    }
}

fn insert_inline_math_tokens(tokens: Vec<Token>, source: &[char]) -> Vec<Token> {
    let mut math_tokens = inline_math_spans(source)
        .into_iter()
        .map(|span| Token::new(span, TokenKind::Unlintable))
        .peekable();

    let mut out = Vec::with_capacity(tokens.len() + math_tokens.size_hint().0);
    for token in tokens {
        while math_tokens
            .peek()
            .is_some_and(|math_token| math_token.span.start < token.span.start)
        {
            out.push(math_tokens.next().unwrap());
        }
        out.push(token);
    }
    out.extend(math_tokens);
    out
}

fn inline_math_spans(source: &[char]) -> Vec<Span<char>> {
    let mut spans = Vec::new();
    let mut cursor = 0;

    while cursor < source.len() {
        if source[cursor] != '$' || is_escaped(source, cursor) {
            cursor += 1;
            continue;
        }

        let Some(relative_end) = source[cursor + 1..].iter().position(|c| *c == '$') else {
            break;
        };
        let end = cursor + 1 + relative_end + 1;
        spans.push(Span::new(cursor, end));
        cursor = end;
    }

    spans
}

fn is_escaped(source: &[char], cursor: usize) -> bool {
    cursor > 0 && source[cursor - 1] == '\\'
}

fn collapse_tex_dashes(tokens: Vec<Token>) -> Vec<Token> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;

    while i < tokens.len() {
        let is_triple_hyphen = i + 2 < tokens.len()
            && matches!(tokens[i].kind, TokenKind::Punctuation(Punctuation::Hyphen))
            && matches!(
                tokens[i + 1].kind,
                TokenKind::Punctuation(Punctuation::Hyphen)
            )
            && matches!(
                tokens[i + 2].kind,
                TokenKind::Punctuation(Punctuation::Hyphen)
            )
            && tokens[i].span.end == tokens[i + 1].span.start
            && tokens[i + 1].span.end == tokens[i + 2].span.start;

        if is_triple_hyphen {
            out.push(Token::new(
                Span::new(tokens[i].span.start, tokens[i + 2].span.end),
                TokenKind::Punctuation(Punctuation::EmDash),
            ));
            i += 3;
            continue;
        }

        let is_double_hyphen = i + 1 < tokens.len()
            && matches!(tokens[i].kind, TokenKind::Punctuation(Punctuation::Hyphen))
            && matches!(
                tokens[i + 1].kind,
                TokenKind::Punctuation(Punctuation::Hyphen)
            )
            && tokens[i].span.end == tokens[i + 1].span.start;

        if is_double_hyphen {
            out.push(Token::new(
                Span::new(tokens[i].span.start, tokens[i + 1].span.end),
                TokenKind::Punctuation(Punctuation::EnDash),
            ));
            i += 2;
            continue;
        }

        out.push(tokens[i].clone());
        i += 1;
    }

    out
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

    #[test]
    fn parses_triple_hyphen_as_em_dash() {
        let source = "A---B";

        let toks = TeX::default().parse_str(source);
        let tok_kinds: Vec<_> = toks.into_iter().map(|t| t.kind).collect();

        assert_eq!(
            tok_kinds,
            vec![
                TokenKind::Word(None),
                TokenKind::Punctuation(harper_core::Punctuation::EmDash),
                TokenKind::Word(None),
            ]
        )
    }

    #[test]
    fn parses_double_hyphen_as_en_dash() {
        let source = "A--B";

        let toks = TeX::default().parse_str(source);
        let tok_kinds: Vec<_> = toks.into_iter().map(|t| t.kind).collect();

        assert_eq!(
            tok_kinds,
            vec![
                TokenKind::Word(None),
                TokenKind::Punctuation(harper_core::Punctuation::EnDash),
                TokenKind::Word(None),
            ]
        )
    }
}
