use harper_core::Lrc;
use harper_core::Token;
use harper_core::parsers::{LineClass, LineWise, MarkdownOptions, Parser};

use super::jsdoc::JsDoc;
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
        Self::new(Lrc::new(JsDoc::new_markdown(markdown_options)))
    }
}

impl Parser for Solidity {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        LineWise::new(self.inner.clone(), classify as fn(&[char]) -> LineClass).parse(source)
    }
}

fn classify(line: &[char]) -> LineClass {
    let actual = without_initiators(line);
    if actual.is_empty() {
        return LineClass::skip();
    }

    let actual_source = actual.get_content(line);

    // Ignore the special SPDX-License-Identifier comment. `line` has
    // already been split on '\n' by `LineWise`, so it never contains
    // one - this always swallows the whole line.
    if actual_source.starts_with(&['S', 'P', 'D', 'X', '-']) {
        return LineClass::skip();
    }

    LineClass::parse(actual)
}
