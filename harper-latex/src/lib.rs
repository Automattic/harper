use itertools::Itertools;
// TODO: remove direct dependency on rowan if possible
// so we don't have to manually match its version with the one the texlab crates use
use rowan::WalkEvent;

use parser::{parse_latex, SyntaxConfig};
use syntax::latex::{SyntaxKind, SyntaxNode};

use harper_core::{
    parsers::{Parser, PlainEnglish, StrParser},
    Token,
};

/// A parser that wraps Harper's `PlainEnglish` parser allowing one to ingest TeX files.
pub struct Tex;

impl Parser for Tex {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let source_str: String = source.iter().collect();

        let latex_document = parse_latex(source_str.as_str(), &SyntaxConfig::default());
        let latex_ast = SyntaxNode::new_root(latex_document);

        let harper_tokens: Vec<_> = latex_ast
            .preorder()
            .into_iter()
            .filter_map(|evt| match evt {
                WalkEvent::Enter(node) => Some(match node.kind() {
                    SyntaxKind::TEXT => PlainEnglish
                        .parse_str(String::from(node.text()).as_str())
                        .into_iter()
                        .map(|mut t| {
                            t.span.push_by(node.index());
                            t
                        })
                        .collect_vec(),
                    // TODO
                    _ => vec![],
                }),
                WalkEvent::Leave(_) => None,
            })
            .flatten()
            .collect();

        dbg!(&harper_tokens);

        harper_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harper_core::parsers::StrParser;

    #[test]
    fn basic() {
        Tex.parse_str(
            r#"
            \documentclass{article}

            \begin{document}
                This is a sentence.

                \section{Section}

                Here is another sentence.
            \end{document}
        "#,
        );
    }
}
