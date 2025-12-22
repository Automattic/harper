use harper_core::Token;
use harper_core::parsers::{self, Markdown, MarkdownOptions, Parser};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct JJDescriptionParser {
    /// Used to grab the text nodes, and parse them as markdown.
    inner: parsers::Mask<TreeSitterMasker, Markdown>,
}

impl JJDescriptionParser {
    fn node_condition(n: &Node) -> bool {
        n.kind() == "text"
    }

    pub fn new(markdown_options: MarkdownOptions) -> Self {
        Self {
            inner: parsers::Mask::new(
                TreeSitterMasker::new(tree_sitter_jjdescription::language(), Self::node_condition),
                Markdown::new(markdown_options),
            ),
        }
    }
}

impl Parser for JJDescriptionParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}
