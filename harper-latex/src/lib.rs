use itertools::Itertools;

use parser::{SyntaxConfig, parse_latex};
use syntax::latex::{SyntaxKind, SyntaxNode};

use harper_core::{
    Token,
    parsers::{Parser, PlainEnglish, StrParser},
};

/// A parser that wraps Harper's `PlainEnglish` parser allowing one to ingest LaTeX files.
pub struct Latex;

impl Parser for Latex {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let source_str: String = source.iter().collect();

        let latex_document = parse_latex(source_str.as_str(), &SyntaxConfig::default());
        let latex_ast = SyntaxNode::new_root(latex_document);

        let harper_tokens: Vec<_> = latex_ast
            .descendants()
            .filter_map(|node| match node.kind() {
                SyntaxKind::TEXT => {
                    // dbg!(&node.text());

                    Some(
                        PlainEnglish
                            .parse_str(String::from(node.text()).as_str())
                            .into_iter()
                            .map(|mut t| {
                                t.span.push_by(node.text_range().start().into());
                                t
                            })
                            .collect_vec(),
                    )
                }
                _ => None,
            })
            .flatten()
            .collect();

        // dbg!(&harper_tokens);

        harper_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harper_core::parsers::StrParser;

    #[test]
    fn basic() {
        Latex.parse_str(
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
