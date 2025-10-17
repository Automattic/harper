use harper_core::Token;
use harper_core::parsers::{self, Markdown, MarkdownOptions, Parser};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct GitCommitParser {
    /// Used to grab the text nodes, and parse them as markdown.
    inner: parsers::Mask<TreeSitterMasker, Markdown>,
}

impl GitCommitParser {
    fn node_condition(n: &Node) -> bool {
        matches!(n.kind(), "subject" | "message_line" | "breaking_change")
    }

    pub fn new(markdown_options: MarkdownOptions) -> Self {
        Self {
            inner: parsers::Mask::new(
                TreeSitterMasker::new(tree_sitter_gitcommit::language(), Self::node_condition),
                Markdown::new(markdown_options),
            ),
        }
    }
}

impl Parser for GitCommitParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}
