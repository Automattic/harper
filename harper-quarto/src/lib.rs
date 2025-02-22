use harper_core::{
    Lrc, Token,
    parsers::{Markdown, MarkdownOptions, Mask, Parser},
};

mod masker;
pub use masker::QuartoMasker;

/// Parses a Literate Haskell document by masking out the code and considering text as Markdown.
pub struct QuartoParser {
    inner: Lrc<dyn Parser>,
}

impl QuartoParser {
    pub fn new(inner: Lrc<dyn Parser>) -> Self {
        Self { inner }
    }

    pub fn new_markdown(markdown_options: MarkdownOptions) -> Self {
        Self {
            inner: Lrc::new(Markdown::new(markdown_options)),
        }
    }
}

impl Parser for QuartoParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        Mask::new(QuartoMasker, self.inner.clone()).parse(source)
    }
}
