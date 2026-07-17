use harper_core::Lrc;
use harper_core::Token;
use harper_core::parsers::{Markdown, MarkdownOptions, Parser};

use super::without_initiators;

#[derive(Clone)]
pub struct Go {
    inner: Lrc<dyn Parser>,
}

impl Go {
    pub fn new(parser: Lrc<dyn Parser>) -> Self {
        Self { inner: parser }
    }

    pub fn new_markdown(markdown_options: MarkdownOptions) -> Self {
        Self::new(Lrc::new(Markdown::new(markdown_options)))
    }
}

/// Whether a comment line is a build constraint or a `go:` directive rather than prose.
///
/// `// +build` is the pre-1.17 spelling that `//go:build` replaced; files keeping
/// compatibility still carry both.
fn is_directive(line: &[char]) -> bool {
    matches!(line, ['g', 'o', ':', ..]) || matches!(line, ['+', 'b', 'u', 'i', 'l', 'd', ..])
}

impl Parser for Go {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut actual = without_initiators(source);

        // Skip leading machine-readable lines. `merge_whitespace_sep` hands us a
        // whole comment block, so a `//go:build` file preamble arrives with the
        // package doc merged in beneath it and there can be several in a row
        // (`//go:build` is conventionally paired with a legacy `// +build`).
        while is_directive(actual.get_content(source)) {
            let Some(offset) = source[actual.start..actual.end]
                .iter()
                .position(|c| *c == '\n')
            else {
                return Vec::new();
            };

            // `actual` indexes `source`, so re-anchor there rather than in the
            // already-stripped slice. This strictly advances, so it terminates.
            actual.start += offset + 1;
            actual.start += without_initiators(actual.get_content(source)).start;
        }

        let Some(actual_source) = actual.try_get_content(source) else {
            return Vec::new();
        };

        let mut new_tokens = self.inner.parse(actual_source);

        new_tokens
            .iter_mut()
            .for_each(|t| t.span.push_by(actual.start));

        new_tokens
    }
}
