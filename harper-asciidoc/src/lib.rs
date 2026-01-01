use harper_core::parsers::{self, Parser, PlainEnglish};
use harper_core::{Token, TokenKind};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct AsciidocParser {
    inner: parsers::Mask<TreeSitterMasker, PlainEnglish>,
}

impl AsciidocParser {
    fn node_condition(n: &Node) -> bool {
        matches!(n.kind(), "line" | "body" | "table_cell_content")
    }
}

impl Default for AsciidocParser {
    fn default() -> Self {
        Self {
            inner: parsers::Mask::new(
                TreeSitterMasker::new(tree_sitter_asciidoc::language().into(), Self::node_condition),
                PlainEnglish,
            ),
        }
    }
}

impl Parser for AsciidocParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut tokens = self.inner.parse(source);

        for token in &mut tokens {
            if let TokenKind::Space(v) = &mut token.kind {
                *v = (*v).clamp(0, 1);
            }
        }

        tokens
    }
}
