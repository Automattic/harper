use harper_core::parsers::{self, Parser, PlainEnglish};
use harper_core::{Token, TokenKind};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct HtmlParser {
    /// Used to grab the text nodes.
    inner: parsers::Mask<TreeSitterMasker, PlainEnglish>,
}

impl HtmlParser {
    fn node_condition(n: &Node) -> bool {
        n.kind() == "text"
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self {
            inner: parsers::Mask::new(
                TreeSitterMasker::new(tree_sitter_html::LANGUAGE.into(), Self::node_condition),
                PlainEnglish,
            ),
        }
    }
}

impl Parser for HtmlParser {
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
