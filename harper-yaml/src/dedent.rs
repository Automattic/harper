use harper_core::parsers::Parser;
use harper_core::{Span, Token, TokenKind};

/// Wraps a `Parser`, parsing each line of a (possibly multiline) span
/// independently after trimming its leading/trailing whitespace, then
/// stitching the results back together with a single explicit
/// [`TokenKind::Newline`] token between lines.
///
/// This exists for YAML block/folded scalars: their continuation lines
/// carry literal indentation (e.g. two leading spaces). Feeding
/// that raw text straight to a prose parser makes the newline-plus-
/// indentation read as a run of several space characters, which
/// incorrectly triggers Harper's "double space" formatting rule.
/// De-denting each line before parsing avoids that, without needing to
/// understand YAML's specific indentation rules.
pub(crate) struct DedentLines<P> {
    inner: P,
}

impl<P: Parser> DedentLines<P> {
    pub(crate) fn new(inner: P) -> Self {
        Self { inner }
    }
}

impl<P: Parser> Parser for DedentLines<P> {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars_traversed = 0;

        for line in source.split(|c| *c == '\n') {
            if let Some(trimmed) = trim(line) {
                let mut new_tokens = self.inner.parse(trimmed.get_content(line));

                new_tokens
                    .iter_mut()
                    .for_each(|t| t.span.push_by(chars_traversed + trimmed.start));

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

/// The span of `line` with leading/trailing whitespace trimmed, or
/// `None` if the line is entirely whitespace.
fn trim(line: &[char]) -> Option<Span<char>> {
    let start = line.iter().position(|c| !c.is_whitespace())?;
    let end = line.len() - line.iter().rev().position(|c| !c.is_whitespace())?;

    Some(Span::new(start, end))
}

#[cfg(test)]
mod tests {
    use harper_core::TokenKind;
    use harper_core::parsers::{Parser, PlainEnglish};

    use super::DedentLines;

    fn parse(source: &str) -> Vec<harper_core::Token> {
        let chars: Vec<char> = source.chars().collect();
        DedentLines::new(PlainEnglish).parse(&chars)
    }

    #[test]
    fn dedents_indented_continuation_line() {
        let source = "hello\n  world";
        let tokens = parse(source);

        // No token should be a run of more than one space - the two
        // leading spaces before "world" must have been trimmed away.
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
}
