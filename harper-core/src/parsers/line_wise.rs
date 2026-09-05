use super::Parser;
use crate::{LSend, Span, Token, TokenKind};

/// Describes how [`LineWise`] should handle a single line.
pub struct LineClass {
    /// The sub-span of the line to hand to the inner parser. An empty
    /// span means nothing on this line gets parsed.
    pub span: Span<char>,
    /// The weight of the `Newline` token to insert after this line (if
    /// there is a following line), or `None` to insert no separator
    /// token at all — e.g. to fully swallow a line inside a fenced code
    /// block.
    pub newline: Option<usize>,
}

impl LineClass {
    /// Parse `span`, followed by a normal single-weight newline separator.
    pub fn parse(span: Span<char>) -> Self {
        Self {
            span,
            newline: Some(1),
        }
    }

    /// Skip the line's content, but still insert a normal newline
    /// separator (e.g. a blank line).
    pub fn skip() -> Self {
        Self {
            span: Span::new(0, 0),
            newline: Some(1),
        }
    }

    /// Skip the line's content and suppress the following separator
    /// entirely.
    pub fn skip_silently() -> Self {
        Self {
            span: Span::new(0, 0),
            newline: None,
        }
    }

    /// Skip the line's content, inserting a custom-weight newline
    /// separator (e.g. a paragraph break).
    pub fn skip_with_newline(weight: usize) -> Self {
        Self {
            span: Span::new(0, 0),
            newline: Some(weight),
        }
    }
}

/// Wraps a `Parser`, parsing a (possibly multiline) span one line at a
/// time. Each line is first reduced by `classify` to a [`LineClass`]
/// describing which sub-span (if any) should be parsed, and what
/// [`TokenKind::Newline`] separator (if any) should follow it — e.g.
/// trimming whitespace or comment initiators, or overriding the
/// separator to mark a paragraph break or swallow a line entirely.
/// Per-line results are stitched back together, with spans remapped
/// back onto the original, untrimmed source.
pub struct LineWise<P, S> {
    inner: P,
    classify: S,
}

impl<P, S> LineWise<P, S>
where
    P: Parser,
    S: Fn(&[char]) -> LineClass + LSend,
{
    pub fn new(inner: P, classify: S) -> Self {
        Self { inner, classify }
    }
}

impl<P, S> Parser for LineWise<P, S>
where
    P: Parser,
    S: Fn(&[char]) -> LineClass + LSend,
{
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars_traversed = 0;

        for line in source.split(|c| *c == '\n') {
            let class = (self.classify)(line);

            if !class.span.is_empty() {
                let mut new_tokens = self.inner.parse(class.span.get_content(line));

                new_tokens
                    .iter_mut()
                    .for_each(|t| t.span.push_by(chars_traversed + class.span.start));

                tokens.append(&mut new_tokens);
            }

            let line_end = chars_traversed + line.len();

            if line_end < source.len()
                && let Some(weight) = class.newline
            {
                tokens.push(Token::new(
                    Span::new_with_len(line_end, 1),
                    TokenKind::Newline(weight),
                ));
            }

            chars_traversed += line.len() + 1;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::{LineClass, LineWise};
    use crate::Span;
    use crate::TokenKind;
    use crate::parsers::{Parser, PlainEnglish};

    fn trim_whitespace(line: &[char]) -> LineClass {
        let Some(start) = line.iter().position(|c| !c.is_whitespace()) else {
            return LineClass::skip();
        };
        let end = line.len() - line.iter().rev().position(|c| !c.is_whitespace()).unwrap();

        LineClass::parse(Span::new(start, end))
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
    fn classify_function_is_not_limited_to_whitespace() {
        // A classify function that chops a leading "> " blockquote marker,
        // proving the combinator generalizes beyond whitespace-trimming.
        fn strip_quote_marker(line: &[char]) -> LineClass {
            if line.starts_with(&['>', ' ']) {
                LineClass::parse(Span::new(2, line.len()))
            } else {
                LineClass::parse(Span::new(0, line.len()))
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

    #[test]
    fn skip_silently_suppresses_the_separator_token() {
        fn classify(line: &[char]) -> LineClass {
            if line == ['-', '-', '-'] {
                LineClass::skip_silently()
            } else {
                LineClass::parse(Span::new(0, line.len()))
            }
        }

        let source = "hello\n---\nworld";
        let chars: Vec<char> = source.chars().collect();
        let tokens = LineWise::new(PlainEnglish, classify).parse(&chars);

        let newline_count = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Newline(_)))
            .count();

        // Only the separator after "hello" should survive - the "---"
        // line's own trailing separator is suppressed by skip_silently,
        // so a naive implementation (2 separators) would fail this.
        assert_eq!(newline_count, 1, "unexpected separator count: {tokens:?}");
    }

    #[test]
    fn skip_with_newline_overrides_the_separator_weight() {
        fn classify(line: &[char]) -> LineClass {
            if line.starts_with(&['@']) {
                LineClass::skip_with_newline(2)
            } else {
                LineClass::parse(Span::new(0, line.len()))
            }
        }

        let source = "hello\n@tag\nworld";
        let chars: Vec<char> = source.chars().collect();
        let tokens = LineWise::new(PlainEnglish, classify).parse(&chars);

        let newline_weights: Vec<usize> = tokens
            .iter()
            .filter_map(|t| match t.kind {
                TokenKind::Newline(n) => Some(n),
                _ => None,
            })
            .collect();

        // "hello"'s own (default-weight) separator comes first, followed
        // by "@tag"'s overridden weight-2 separator.
        assert_eq!(newline_weights, vec![1, 2]);
    }

    #[test]
    fn skip_with_newline_on_the_final_line_emits_no_trailing_separator() {
        fn classify(line: &[char]) -> LineClass {
            if line.starts_with(&['@']) {
                LineClass::skip_with_newline(2)
            } else {
                LineClass::parse(Span::new(0, line.len()))
            }
        }

        // "@tag" is the only, and therefore final, line - its requested
        // separator weight must not produce a dangling trailing token.
        let source = "@tag";
        let chars: Vec<char> = source.chars().collect();
        let tokens = LineWise::new(PlainEnglish, classify).parse(&chars);

        assert!(
            tokens.is_empty(),
            "did not expect any tokens for a swallowed final line: {tokens:?}"
        );
    }
}
