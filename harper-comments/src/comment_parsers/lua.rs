use harper_core::Lrc;
use harper_core::Token;
use harper_core::parsers::{LineClass, LineWise, Markdown, MarkdownOptions, Parser};

use super::without_initiators;

#[derive(Clone)]
pub struct Lua {
    inner: Lrc<dyn Parser>,
}

impl Lua {
    pub fn new(parser: Lrc<dyn Parser>) -> Self {
        Self { inner: parser }
    }

    pub fn new_markdown(markdown_options: MarkdownOptions) -> Self {
        Self::new(Lrc::new(Markdown::new(markdown_options)))
    }
}

impl Parser for Lua {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        LineWise::new(self.inner.clone(), classify as fn(&[char]) -> LineClass).parse(source)
    }
}

fn classify(line: &[char]) -> LineClass {
    let actual = without_initiators(line);
    let actual_chars = actual.get_content(line);

    // A `@tag` annotation line (e.g. LuaDoc's `---@param`) is never
    // parsed as prose; it's treated as its own paragraph break so it
    // doesn't get glued onto the surrounding sentence.
    if matches!(actual_chars, ['@', ..]) {
        return LineClass::skip_with_newline(2);
    }

    if actual.is_empty() {
        return LineClass::skip();
    }

    LineClass::parse(actual)
}
