use crate::{Span, Token, TokenKind};

use super::Parser;

/// A parser that wraps another, forcing the entirety of the document to be composed of headings.
pub struct OopsAllHeadings<P: Parser + 'static> {
    inner: P,
}

impl<P: Parser + 'static> OopsAllHeadings<P> {
    pub fn new(inner: P) -> Self {
        Self { inner }
    }
}

impl<P: Parser + 'static> Parser for OopsAllHeadings<P> {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let inner = self.inner.parse(source);
        let mut output = Vec::with_capacity(inner.capacity());

        output.push(Token {
            span: Span::default(),
            kind: TokenKind::HeadingStart,
        });

        let mut iter = inner.into_iter().peekable();

        while let Some(tok) = iter.next() {
            let heading_start = if tok.kind.is_paragraph_break()
                && iter.peek().is_some_and(|t| !t.kind.is_heading_start())
            {
                Some(Token {
                    span: Span::empty(tok.span.end),
                    kind: TokenKind::HeadingStart,
                })
            } else {
                None
            };

            output.push(tok);

            if let Some(extra) = heading_start {
                output.push(extra);
            }
        }

        output
    }
}
