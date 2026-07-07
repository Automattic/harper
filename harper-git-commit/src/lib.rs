use harper_core::parsers::{self, Markdown, MarkdownOptions, Parser};
use harper_core::{Token, TokenKind};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct GitCommitParser {
    inner: parsers::Mask<TreeSitterMasker, Markdown>,
}

impl GitCommitParser {
    fn node_condition(n: &Node) -> bool {
        matches!(n.kind(), "subject" | "message_line" | "breaking_change")
    }

    pub fn new(markdown_options: MarkdownOptions) -> Self {
        Self {
            inner: parsers::Mask::new(
                TreeSitterMasker::new(tree_sitter_gitcommit::LANGUAGE.into(), Self::node_condition),
                Markdown::new(markdown_options),
            ),
        }
    }
}

impl Default for GitCommitParser {
    fn default() -> Self {
        Self::new(MarkdownOptions::default())
    }
}

impl Parser for GitCommitParser {
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
