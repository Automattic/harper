use itertools::Itertools;

use parser::{SyntaxConfig, parse_latex};
use syntax::latex::{SyntaxKind, SyntaxNode};

use harper_core::{
    Punctuation, Span, Token, TokenKind,
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

        let mut harper_tokens_initial = latex_ast
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
            .collect_vec();

        // dummy token to allow counting consecutive hyphens at the right edge
        harper_tokens_initial.push(Token::new(Span::new(0, 0), TokenKind::Unlintable));

        let mut consecutive_hyphens = 0;
        let mut consecutive_spaces = 0;

        let mut harper_tokens: Vec<Token> = vec![];
        for mut token in harper_tokens_initial {
            if matches!(token.kind, TokenKind::Newline(1)) {
                token.kind = TokenKind::Space(1);
            }

            if matches!(token.kind, TokenKind::Space(_)) {
                token.kind = TokenKind::Space(1);

                consecutive_spaces += 1;
            } else if consecutive_spaces > 1 {
                let mut spaces = vec![];
                for _ in 0..consecutive_spaces {
                    spaces.push(harper_tokens.pop().unwrap());
                }
                let mut total_span = spaces.first().expect("at least two").span;
                for h in &spaces[1..] {
                    total_span.expand_to_include(h.span.end);
                }

                harper_tokens.push(Token {
                    span: total_span,
                    kind: TokenKind::Space(1),
                });

                consecutive_spaces = 0;
            } else {
                consecutive_spaces = 0;
            }

            if matches!(token.kind, TokenKind::Punctuation(_))
                && token.span.get_content_string(source) == "~"
            {
                // non-breaking space
                token.kind = TokenKind::Space(1);
            }

            if matches!(token.kind, TokenKind::Punctuation(Punctuation::Hyphen)) {
                consecutive_hyphens += 1;
            } else if consecutive_hyphens == 2 || consecutive_hyphens == 3 {
                let mut hyphens = vec![];
                for _ in 0..consecutive_hyphens {
                    hyphens.push(harper_tokens.pop().unwrap());
                }
                let mut total_span = hyphens.first().expect("at least two").span;
                for h in &hyphens[1..] {
                    total_span.expand_to_include(h.span.end);
                }

                harper_tokens.push(Token {
                    span: total_span,
                    kind: TokenKind::Punctuation(match consecutive_hyphens {
                        2 => Punctuation::EnDash,
                        3 => Punctuation::EmDash,
                        _ => unreachable!("already narrowed"),
                    }),
                });

                consecutive_hyphens = 0;
            } else {
                consecutive_hyphens = 0;
            }

            harper_tokens.push(token);
        }

        harper_tokens
            .pop()
            .expect("it will have at least the dummy token");

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
    fn consecutive_spaces() {
        let source = r#"a      b"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[1].kind, TokenKind::Space(1)));
    }

    #[test]
    fn newline_then_indent() {
        let source = r#"some
        stuff"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[1].kind, TokenKind::Space(1)));
    }

    #[test]
    fn en_dash() {
        let source = r#"6--7"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn em_dash() {
        let source = r#"---"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        assert_eq!(tokens.len(), 1);
    }

    #[test]
    #[ignore]
    fn double_quotes() {
        let source = r#"``stuff''"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        assert_eq!(tokens.len(), 3);
    }

    #[test]
    #[ignore]
    fn single_quotes() {
        let source = r#"`stuff'"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        assert_eq!(tokens.len(), 3);
    }

    #[test]
    #[ignore]
    fn apostrophe() {
        let source = r#"The book's cover"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        assert_eq!(tokens.len(), 7);
    }

    #[test]
    fn non_breaking_space() {
        let source = r#"This~that"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[1].kind, TokenKind::Space(1)));
    }

    #[test]
    #[ignore]
    fn comment() {
        let source = r#"% A comment"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn multi_byte_chars() {
        let source = r#"An errorz."#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();

        assert_eq!(tokens.len(), 4);

        assert_eq!(
            tokens[3]
                .to_fat(source.chars().collect_vec().as_ref())
                .content,
            "errorz".chars().collect_vec()
        );

        let lens: [usize; _] = [2, 1, 6, 1];
        lens.into_iter().enumerate().for_each(|(i, len)| {
            let token = &tokens[i];
            assert_eq!(token.span.end - token.span.start, len);
        });
    }
}
