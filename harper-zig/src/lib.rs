use harper_core::Token;
use harper_core::parsers::{self, Parser, PlainEnglish};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct ZigParser {
    /// Used to grab the comment nodes, and parse them as plain English.
    inner: parsers::Mask<TreeSitterMasker, PlainEnglish>,
}

impl ZigParser {
    fn node_condition(n: &Node) -> bool {
        matches!(n.kind(), "comment")
    }

    pub fn new() -> Self {
        Self {
            inner: parsers::Mask::new(
                TreeSitterMasker::new(tree_sitter_zig::LANGUAGE.into(), Self::node_condition),
                PlainEnglish,
            ),
        }
    }
}

impl Default for ZigParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for ZigParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}
