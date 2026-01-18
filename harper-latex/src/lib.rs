use std::collections::HashMap;

use itertools::Itertools;
use regex::Regex;

use parser::{SyntaxConfig, parse_latex};
use syntax::latex::{SyntaxKind, SyntaxNode};

use harper_core::{
    Punctuation, Quote, Span, Token, TokenKind,
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

        let mut heading_open = false; // jank (use sane ast traversal instead?)
        let mut harper_tokens_initial = latex_ast
            .descendants_with_tokens()
            .filter_map(|node| {
                match node.kind() {
                    SyntaxKind::TEXT => Some(text_node_to_tokens(
                        node.into_node().unwrap(),
                        &byte_to_char,
                    )),
                    SyntaxKind::SECTION | SyntaxKind::SUBSECTION | SyntaxKind::SUBSUBSECTION => {
                        heading_open = true;

                        let [span_start, span_end] = [
                            u32::from(node.text_range().start()) + 1,
                            u32::from(
                                node.into_node()
                                    .unwrap()
                                    .first_child()
                                    .unwrap()
                                    .text_range()
                                    .start(),
                            ), // re-indexing not necessary?
                        ]
                        .map(|p| byte_to_char[p as usize] as usize);

                        Some(vec![Token {
                            span: Span::new(span_start, span_end),
                            kind: TokenKind::HeadingStart,
                        }])
                    }
                    SyntaxKind::R_CURLY => {
                        if heading_open {
                            heading_open = false;

                            Some(vec![Token {
                                span: Span::new(
                                    (u32::from(node.text_range().start()) + 1) as usize,
                                    (u32::from(node.text_range().end()) + 1) as usize,
                                ),
                                kind: TokenKind::ParagraphBreak,
                            }])
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
            .flatten()
            .collect_vec();

        post_process_tokens(source, &mut harper_tokens_initial)
    }
}

fn post_process_tokens(source: &[char], harper_tokens_initial: &mut Vec<Token>) -> Vec<Token> {
    // dummy token to allow counting consecutive hyphens at the right edge
    harper_tokens_initial.push(Token::new(Span::new(0, 0), TokenKind::Unlintable));

    let mut consecutive_hyphens = 0;
    let mut consecutive_spaces = 0;

    let mut harper_tokens: Vec<Token> = vec![];
    for token in harper_tokens_initial {
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

        harper_tokens.push(token.clone());
    }

    harper_tokens
        .pop()
        .expect("it will have at least the dummy token");

    harper_tokens
}

fn text_node_to_tokens(node: SyntaxNode, byte_to_char: &[u32]) -> Vec<Token> {
    let text = String::from(node.text());

    let harper_tokens = PlainEnglish
        .parse_str(&text)
        .into_iter()
        .map(|mut t| {
            let span_start = byte_to_char[u32::from(node.text_range().start()) as usize];
            t.span.push_by(span_start as usize);
            t
        })
        .collect_vec();

    let quotes_re = Regex::new(r"``?|''?").unwrap();
    let quotes_by_start: HashMap<usize, _> =
        HashMap::from_iter(quotes_re.find_iter(&text).map(|m| (m.start(), m)));

    let mut single_quote_open_stack: Vec<usize> = vec![]; // indices of currently open quote tokens
    let mut double_quote_open_stack: Vec<usize> = vec![];

    let mut harper_tokens_mod = vec![];
    let mut i = 0;

    while i < harper_tokens.len() {
        let t = harper_tokens.get(i).unwrap();
        let start = t.span.start;

        harper_tokens_mod.push(t.clone());

        if quotes_by_start.contains_key(&start) {
            let m = quotes_by_start.get(&start).unwrap();
            let match_text = m.as_str();

            let is_double = match_text == "``" || match_text == "''";
            let is_open = match_text == "``" || match_text == "`";

            if !is_double && !is_open && single_quote_open_stack.is_empty() {
                i += 1;
                continue; // it's probably just an apostrophe
            }

            let open_stack = if is_double {
                &mut double_quote_open_stack
            } else {
                &mut single_quote_open_stack
            };

            let corresponding_open_idx = open_stack.pop();

            let twin_loc = if is_open {
                None
            } else {
                let corresponding_open = harper_tokens_mod
                    .get(corresponding_open_idx.unwrap())
                    .unwrap();
                Some(corresponding_open.span.start)
            };

            let quote_token = harper_tokens_mod.last_mut().unwrap();
            quote_token.kind = TokenKind::Punctuation(Punctuation::Quote(Quote { twin_loc }));

            if is_double {
                quote_token.span.end += 1;
            }

            if is_open {
                open_stack.push(i);
            } else {
                let corresponding_open = harper_tokens_mod
                    .get_mut(corresponding_open_idx.unwrap())
                    .unwrap();
                corresponding_open.kind = TokenKind::Punctuation(Punctuation::Quote(Quote {
                    twin_loc: Some(start),
                }));
            }

            if is_double {
                i += 1; // skip the next backtick/apostrophe token
            }
        }

        i += 1;
    }

    harper_tokens_mod
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
    fn double_quotes() {
        let source = r#"``stuff''"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        let (open_start, close_start) = (0, 7);
        let (open, close) = (&tokens[0], &tokens[2]);

        assert_eq!(tokens.len(), 3);
        assert_eq!(
            open.kind,
            TokenKind::Punctuation(Punctuation::Quote(Quote {
                twin_loc: Some(close_start)
            }))
        );
        assert_eq!(open.span.end - open.span.start, 2);
        assert_eq!(open.span.start, open_start);
        assert_eq!(open.span.end, open_start + 2);

        assert_eq!(
            close.kind,
            TokenKind::Punctuation(Punctuation::Quote(Quote {
                twin_loc: Some(open_start)
            }))
        );
        assert_eq!(close.span.end - close.span.start, 2);
        assert_eq!(close.span.start, close_start);
        assert_eq!(close.span.end, close_start + 2);
    }

    #[test]
    fn single_quotes() {
        let source = r#"`stuff'"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        let (open_start, close_start) = (0, 6);
        let (open, close) = (&tokens[0], &tokens[2]);

        assert_eq!(tokens.len(), 3);
        assert_eq!(
            open.kind,
            TokenKind::Punctuation(Punctuation::Quote(Quote {
                twin_loc: Some(close_start)
            }))
        );
        assert_eq!(open.span.end - open.span.start, 1);
        assert_eq!(open.span.start, open_start);
        assert_eq!(open.span.end, open_start + 1);

        assert_eq!(
            close.kind,
            TokenKind::Punctuation(Punctuation::Quote(Quote {
                twin_loc: Some(open_start)
            }))
        );
        assert_eq!(close.span.end - close.span.start, 1);
        assert_eq!(close.span.start, close_start);
        assert_eq!(close.span.end, close_start + 1);
    }

    #[test]
    fn apostrophe() {
        let source = r#"The book's cover"#;

        let document = Document::new_curated(source, &Latex);
        assert!(!document.tokens().any(|t| t.kind.is_quote()));

        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);
    }

    #[test]
    fn apostrophe_in_single_quote() {
        let source = r#"`It's not'"#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);
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
        let source = r#"An “errorz”."#; // this is not very latex but that's not the point

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

    #[test]
    fn section() {
        let source = r#"
            \section{Section}

            Words, words.
        "#;

        let document = Document::new_curated(source, &Latex);
        let tokens = document.tokens().map(|t| t.clone()).collect_vec();
        dbg!(&tokens);

        assert!(tokens[0].kind.is_heading_start());
        assert_eq!(tokens[0].span.end + 1, tokens[1].span.start);
        assert!(tokens[1].kind.is_word());
        assert!(tokens[2].kind.is_paragraph_break());
    }
}
