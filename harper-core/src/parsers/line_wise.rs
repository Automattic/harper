use super::Parser;
use crate::{LSend, Span, Token, TokenKind};

/// Wraps a `Parser`, parsing a (possibly multiline) span one line at a
/// time. Each line is first reduced by `strip` to the sub-span that
/// should actually be parsed (e.g. trimming whitespace or comment
/// initiators) — an empty span means the line is skipped entirely.
/// Per-line results are stitched back together with a single explicit
/// [`TokenKind::Newline`] token between lines, with spans remapped
/// back onto the original, untrimmed source.
pub struct LineWise<P, S> {
    inner: P,
    strip: S,
}

impl<P, S> LineWise<P, S>
where
    P: Parser,
    S: Fn(&[char]) -> Span<char> + LSend,
{
    pub fn new(inner: P, strip: S) -> Self {
        Self { inner, strip }
    }
}

impl<P, S> Parser for LineWise<P, S>
where
    P: Parser,
    S: Fn(&[char]) -> Span<char> + LSend,
{
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars_traversed = 0;

        for line in source.split(|c| *c == '\n') {
            let stripped = (self.strip)(line);

            if !stripped.is_empty() {
                let mut new_tokens = self.inner.parse(stripped.get_content(line));

                new_tokens
                    .iter_mut()
                    .for_each(|t| t.span.push_by(chars_traversed + stripped.start));

                tokens.append(&mut new_tokens);
            }

            let line_end = chars_traversed + line.len();

            if line_end < source.len() {
                tokens.push(Token::new(
                    Span::new_with_len(line_end, 1),
                    TokenKind::Newline(1),
                ));
            }

            chars_traversed += line.len() + 1;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::LineWise;
    use crate::Span;
    use crate::TokenKind;
    use crate::parsers::{Parser, PlainEnglish};

    fn trim_whitespace(line: &[char]) -> Span<char> {
        let Some(start) = line.iter().position(|c| !c.is_whitespace()) else {
            return Span::new(0, 0);
        };
        let end = line.len() - line.iter().rev().position(|c| !c.is_whitespace()).unwrap();

        Span::new(start, end)
    }

    fn parse(source: &str) -> Vec<crate::Token> {
        let chars: Vec<char> = source.chars().collect();
        LineWise::new(PlainEnglish, trim_whitespace).parse(&chars)
    }

    #[test]
    fn dedents_indented_continuation_line() {
        let tokens = parse("hello\n  world");

        for token in &tokens {
            if let TokenKind::Space(count) = token.kind {
                assert_eq!(count, 1, "found a multi-space token: {tokens:?}");
            }
        }
    }

    #[test]
    fn inserts_single_newline_between_lines() {
        let tokens = parse("hello\nworld");
        let newline_count = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Newline(_)))
            .count();

        assert_eq!(newline_count, 1);
    }

    #[test]
    fn skips_blank_lines_without_panicking() {
        let tokens = parse("hello\n\nworld");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn spans_map_back_to_original_source() {
        let source = "hello\n  world";
        let chars: Vec<char> = source.chars().collect();
        let tokens = parse(source);

        for token in &tokens {
            assert!(token.span.try_get_content(&chars).is_some());
        }
    }

    #[test]
    fn strip_function_is_not_limited_to_whitespace() {
        // A strip function that chops a leading "> " blockquote marker,
        // proving the combinator generalizes beyond whitespace-trimming.
        fn strip_quote_marker(line: &[char]) -> Span<char> {
            if line.starts_with(&['>', ' ']) {
                Span::new(2, line.len())
            } else {
                Span::new(0, line.len())
            }
        }

        let source = "> hello\n> world";
        let chars: Vec<char> = source.chars().collect();
        let tokens = LineWise::new(PlainEnglish, strip_quote_marker).parse(&chars);

        for token in &tokens {
            let content: String = token.span.get_content(&chars).iter().collect();
            assert!(
                !content.starts_with('>'),
                "marker leaked into token: {content:?}"
            );
        }
    }
}
