use harper_core::Lrc;
use harper_core::Span;
use harper_core::Token;
use harper_core::parsers::{Markdown, MarkdownOptions, Parser};

use super::without_initiators;

#[derive(Clone)]
pub struct Solidity {
    inner: Lrc<dyn Parser>,
}

impl Solidity {
    pub fn new(parser: Lrc<dyn Parser>) -> Self {
        Self { inner: parser }
    }

    pub fn new_markdown(markdown_options: MarkdownOptions) -> Self {
        Self::new(Lrc::new(Markdown::new(markdown_options)))
    }
}

impl Parser for Solidity {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut tokens = Vec::new();

        let mut chars_traversed = 0;

        for line in source.split(|c| *c == '\n') {
            let mut new_tokens = parse_line(line, self.inner.clone());

            if chars_traversed + line.len() < source.len() {
                new_tokens.push(Token::new(
                    Span::new_with_len(line.len(), 1),
                    harper_core::TokenKind::Newline(1),
                ));
            }

            new_tokens
                .iter_mut()
                .for_each(|t| t.span.push_by(chars_traversed));

            chars_traversed += line.len() + 1;
            tokens.append(&mut new_tokens);
        }

        tokens
    }
}

fn parse_line(source: &[char], parser: Lrc<dyn Parser>) -> Vec<Token> {
    let mut actual = without_initiators(source);
    if actual.is_empty() {
        return Vec::new();
    }
    let mut actual_source = actual.get_content(source);

    // ignore the special SPDX-License-Identifier comment
    if actual_source.starts_with(&['S', 'P', 'D', 'X', '-']) {
        let Some(terminator) = source.iter().position(|c| *c == '\n') else {
            return Vec::new();
        };

        actual.start += terminator;

        let Some(new_source) = actual.try_get_content(actual_source) else {
            return Vec::new();
        };

        actual_source = new_source
    }

    let mut new_tokens = parser.parse(actual_source);

    new_tokens
        .iter_mut()
        .for_each(|t| t.span.push_by(actual.start));

    new_tokens
}
