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

        let byte_to_char = make_byte_to_char_mapping(&source_str);

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
                                let span_start =
                                    byte_to_char[u32::from(node.text_range().start()) as usize];
                                t.span.push_by(span_start as usize);
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

fn make_byte_to_char_mapping(source_str: &str) -> Vec<u32> {
    let mut byte_to_char = vec![0; source_str.len() + 1];
    let mut char_index = 0u32;
    let mut byte_idx = 0;
    for ch in source_str.chars() {
        let char_len = ch.len_utf8();
        for _ in 0..char_len {
            byte_to_char[byte_idx] = char_index;
            byte_idx += 1;
        }
        char_index += 1;
    }
    byte_to_char[source_str.len()] = char_index;

    byte_to_char
}

#[cfg(test)]
mod tests {
    use super::*;
    use harper_core::{Document, parsers::StrParser};

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

    #[test]
    fn multi_byte_chars() {
        let source = r#"An “errorz”."#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();

        assert_eq!(tokens.len(), 6);

        assert_eq!(
            tokens[3]
                .to_fat(source.chars().collect_vec().as_ref())
                .content,
            "errorz".chars().collect_vec()
        );

        let lens: [usize; _] = [2, 1, 1, 6, 1, 1];
        lens.into_iter().enumerate().for_each(|(i, len)| {
            let token = &tokens[i];
            assert_eq!(token.span.end - token.span.start, len);
        });
    }
}
