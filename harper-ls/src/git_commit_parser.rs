use harper_core::parsers::{Markdown, Parser};

/// A Harper parser for Git commit files
pub struct GitCommitParser;

impl Parser for GitCommitParser {
    /// Admittedly a somewhat naive implementation.
    /// We're going to get _something_ to work, before we polish it off.
    fn parse(&self, source: &[char]) -> Vec<harper_core::Token> {
        // Locate the first `#`
        let end = source
            .iter()
            .position(|c| *c == '#')
            .unwrap_or(source.len());

        Markdown.parse(&source[0..end])
    }
}
