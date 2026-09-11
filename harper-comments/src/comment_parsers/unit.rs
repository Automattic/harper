use std::sync::atomic::{AtomicBool, Ordering};

use harper_core::Lrc;
use harper_core::Token;
use harper_core::parsers::{LineClass, LineWise, Markdown, MarkdownOptions, Parser};

use super::without_initiators;

/// A comment parser that strips starting `/` and `*` characters.
///
/// It is meant to cover _most_ cases in _most_ programming languages.
///
/// It assumes it is being provided a single line of comment at a time,
/// including the comment initiation characters.
#[derive(Clone)]
pub struct Unit {
    inner: Lrc<dyn Parser>,
}

impl Unit {
    pub fn new(parser: Lrc<dyn Parser>) -> Self {
        Self { inner: parser }
    }

    pub fn new_markdown(markdown_options: MarkdownOptions) -> Self {
        Self::new(Lrc::new(Markdown::new(markdown_options)))
    }
}

impl Parser for Unit {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        // Tracks whether we're currently inside a fenced code block,
        // toggled by `classify` as it walks the lines in order. Reset
        // fresh on every `parse` call, since `AtomicBool` (rather than
        // `Cell`) is needed only to satisfy `LineWise`'s `Send + Sync`
        // bound - there's no real concurrency here.
        let in_code_fence = AtomicBool::new(false);

        let classify = move |line: &[char]| -> LineClass {
            if line_is_code_fence(line) {
                in_code_fence.fetch_xor(true, Ordering::Relaxed);
            }

            if in_code_fence.load(Ordering::Relaxed) {
                // Fully swallow lines inside (and the opening marker of)
                // a fenced code block: no content, no separator token.
                return LineClass::skip_silently();
            }

            let actual = without_initiators(line);

            if actual.is_empty() {
                return LineClass::skip();
            }

            LineClass::parse(actual)
        };

        LineWise::new(self.inner.clone(), classify).parse(source)
    }
}

fn line_is_code_fence(source: &[char]) -> bool {
    let actual = without_initiators(source);
    let actual_chars = actual.get_content(source);

    matches!(actual_chars, ['`', '`', '`', ..])
}
