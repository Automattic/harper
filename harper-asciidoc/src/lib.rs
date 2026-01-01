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

#[cfg(test)]
mod tests {
    use super::*;
    use harper_core::parsers::Parser;

    #[test]
    fn parses_basic_text() {
        let parser = AsciidocParser::default();
        let source: Vec<char> = "Hello world".chars().collect();
        let tokens = parser.parse(&source);
        assert!(tokens.iter().any(|t| t.kind.is_word()));
    }

    #[test]
    fn parses_heading() {
        let parser = AsciidocParser::default();
        let source_str = "= Title";
        let source: Vec<char> = source_str.chars().collect();
        let tokens = parser.parse(&source);
        assert!(tokens.iter().any(|t| t.kind.is_word()));
        // Ensure the '=' is not part of any word
        for token in tokens {
            if token.kind.is_word() {
                let word_chars = &source[token.span.start..token.span.end];
                assert!(!word_chars.contains(&'='));
            }
        }
    }

    #[test]
    fn parses_table() {
        let parser = AsciidocParser::default();
        let source_str = "|===\n| Cell 1 | Cell 2\n|===";
        let source: Vec<char> = source_str.chars().collect();
        let tokens = parser.parse(&source);
        assert!(tokens.iter().any(|t| t.kind.is_word()));
    }

    #[test]
    fn parses_comment() {
        let parser = AsciidocParser::default();
        let source: Vec<char> = "// This is a comment".chars().collect();
        let tokens = parser.parse(&source);
        assert!(tokens.iter().any(|t| t.kind.is_word()));
    }
}
