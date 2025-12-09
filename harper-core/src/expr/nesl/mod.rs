mod error;

use is_macro::Is;

use crate::lexing::{FoundToken, lex_nesl_token};
use crate::{CharString, Punctuation, Span, Token, TokenKind};

use error::Error;

struct Ast {
    root: AstNode,
}

#[derive(Debug, Clone, Is, Eq, PartialEq)]
enum AstNode {
    Whitespace,
    Word(CharString),
    Not(Box<AstNode>),
    Seq(Vec<AstNode>),
    Arr(Vec<AstNode>),
}

fn lex(source: &[char]) -> Vec<Token> {
    let mut cursor = 0;

    let mut tokens = Vec::new();

    loop {
        if cursor >= source.len() {
            return tokens;
        }

        if let Some(FoundToken { token, next_index }) = lex_nesl_token(&source[cursor..]) {
            tokens.push(Token {
                span: Span::new(cursor, cursor + next_index),
                kind: token,
            });
            cursor += next_index;
        } else {
            panic!()
        }
    }
}

fn parse_str(nesl: &str) -> Result<Ast, Error> {
    let chars: CharString = nesl.chars().collect();
    let tokens = lex(&chars);

    let mut seq = parse_seq(&tokens, &chars)?;

    // Easy and simple optimizations
    let root = if seq.len() == 1 {
        seq.pop().unwrap()
    } else {
        AstNode::Seq(seq)
    };

    Ok(Ast { root })
}

fn parse_seq(tokens: &[Token], source: &[char]) -> Result<Vec<AstNode>, Error> {
    let mut seq = Vec::new();

    let mut cursor = 0;
    while let Some(remainder) = tokens.get(cursor..)
        && remainder.len() > 0
    {
        let res = parse_node(remainder, source)?;
        seq.push(res.node);
        cursor += res.next_idx;
    }

    Ok(seq)
}

#[derive(Debug)]
struct FoundNode {
    /// The parsed node found.
    node: AstNode,
    /// The next token to be read.
    next_idx: usize,
}

impl FoundNode {
    pub fn new(node: AstNode, next_idx: usize) -> Self {
        Self { node, next_idx }
    }
}

fn parse_node(tokens: &[Token], source: &[char]) -> Result<FoundNode, Error> {
    let tok = tokens.first().ok_or(Error::EndOfInput)?;

    match tok.kind {
        TokenKind::Space(_) => Ok(FoundNode::new(AstNode::Whitespace, 1)),
        TokenKind::Word(_) => Ok(FoundNode::new(
            AstNode::Word(tok.span.get_content(source).into()),
            1,
        )),
        TokenKind::Punctuation(Punctuation::OpenRound) => {
            let close_idx =
                locate_matching_brace(tokens, TokenKind::is_open_round, TokenKind::is_close_round)
                    .ok_or(Error::UnmatchedBrace)?;
            let child = parse_seq(&tokens[1..close_idx], source)?;
            Ok(FoundNode::new(AstNode::Seq(child), close_idx + 1))
        }

        TokenKind::Punctuation(Punctuation::Bang) => {
            let res = parse_node(&tokens[1..], source)?;

            Ok(FoundNode::new(
                AstNode::Not(Box::new(res.node)),
                res.next_idx + 1,
            ))
        }
        TokenKind::Punctuation(Punctuation::OpenSquare) => {
            let close_idx = locate_matching_brace(
                tokens,
                TokenKind::is_open_square,
                TokenKind::is_close_square,
            )
            .ok_or(Error::UnmatchedBrace)?;

            let mut children = Vec::new();

            let mut cursor = 1;

            while cursor < close_idx {
                let new_child = parse_node(&tokens[cursor..close_idx], source)?;
                children.push(new_child.node);

                cursor += new_child.next_idx;

                if cursor != close_idx && !tokens[cursor].kind.is_comma() {
                    dbg!(&tokens[cursor].kind);
                    dbg!(&tokens[cursor].span.get_content_string(source));
                    return Err(Error::ExpectedComma);
                }

                cursor += 1;

                if cursor < close_idx && tokens[cursor].kind.is_space() {
                    cursor += 1;
                }
            }

            Ok(FoundNode::new(AstNode::Arr(children), close_idx + 1))
        }
        _ => {
            dbg!(&tok.kind);
            Err(Error::UnsupportedToken)
        }
    }
}

/// Locates the closing brace for the token at index 0.
fn locate_matching_brace(
    tokens: &[Token],
    is_open: impl Fn(&TokenKind) -> bool,
    is_close: impl Fn(&TokenKind) -> bool,
) -> Option<usize> {
    // Locate closing brace
    let mut visited_opens = 0;
    let mut cursor = 1;

    loop {
        let Some(cur) = tokens.get(cursor) else {
            return None;
        };

        if is_open(&cur.kind) {
            visited_opens += 1;
        } else if is_close(&cur.kind) {
            if visited_opens == 0 {
                return Some(cursor);
            } else {
                visited_opens -= 1;
            }
        }

        cursor += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::char_string::char_string;

    use super::{AstNode, parse_str};

    #[test]
    fn parses_whitespace() {
        assert_eq!(parse_str(" ").unwrap().root, AstNode::Whitespace)
    }

    #[test]
    fn parses_word() {
        assert_eq!(
            parse_str("word").unwrap().root,
            AstNode::Word(char_string!("word"))
        )
    }

    #[test]
    fn parses_word_space() {
        assert_eq!(
            parse_str("word ").unwrap().root,
            AstNode::Seq(vec![
                AstNode::Word(char_string!("word")),
                AstNode::Whitespace
            ])
        )
    }

    #[test]
    fn parses_word_space_word() {
        assert_eq!(
            parse_str("word word").unwrap().root,
            AstNode::Seq(vec![
                AstNode::Word(char_string!("word")),
                AstNode::Whitespace,
                AstNode::Word(char_string!("word")),
            ])
        )
    }

    #[test]
    fn parses_simple_seq() {
        assert_eq!(
            parse_str("a (b c) d").unwrap().root,
            AstNode::Seq(vec![
                AstNode::Word(char_string!("a")),
                AstNode::Whitespace,
                AstNode::Seq(vec![
                    AstNode::Word(char_string!("b")),
                    AstNode::Whitespace,
                    AstNode::Word(char_string!("c")),
                ]),
                AstNode::Whitespace,
                AstNode::Word(char_string!("d")),
            ])
        )
    }

    #[test]
    fn parses_nested_seqs() {
        assert_eq!(
            parse_str("a (b (c)) d").unwrap().root,
            AstNode::Seq(vec![
                AstNode::Word(char_string!("a")),
                AstNode::Whitespace,
                AstNode::Seq(vec![
                    AstNode::Word(char_string!("b")),
                    AstNode::Whitespace,
                    AstNode::Seq(vec![AstNode::Word(char_string!("c")),]),
                ]),
                AstNode::Whitespace,
                AstNode::Word(char_string!("d")),
            ])
        )
    }

    #[test]
    fn parses_paired_seqs() {
        assert_eq!(
            parse_str("a (b) (c) d").unwrap().root,
            AstNode::Seq(vec![
                AstNode::Word(char_string!("a")),
                AstNode::Whitespace,
                AstNode::Seq(vec![AstNode::Word(char_string!("b")),]),
                AstNode::Whitespace,
                AstNode::Seq(vec![AstNode::Word(char_string!("c")),]),
                AstNode::Whitespace,
                AstNode::Word(char_string!("d")),
            ])
        )
    }

    #[test]
    fn parses_not() {
        assert_eq!(
            parse_str("a !b c").unwrap().root,
            AstNode::Seq(vec![
                AstNode::Word(char_string!("a")),
                AstNode::Whitespace,
                AstNode::Not(Box::new(AstNode::Word(char_string!("b")))),
                AstNode::Whitespace,
                AstNode::Word(char_string!("c")),
            ])
        )
    }

    #[test]
    fn parses_not_seq() {
        assert_eq!(
            parse_str("a !(b c) d").unwrap().root,
            AstNode::Seq(vec![
                AstNode::Word(char_string!("a")),
                AstNode::Whitespace,
                AstNode::Not(Box::new(AstNode::Seq(vec![
                    AstNode::Word(char_string!("b")),
                    AstNode::Whitespace,
                    AstNode::Word(char_string!("c")),
                ]),)),
                AstNode::Whitespace,
                AstNode::Word(char_string!("d")),
            ])
        )
    }

    #[test]
    fn parses_empty_array() {
        assert_eq!(parse_str("[]").unwrap().root, AstNode::Arr(vec![]))
    }

    #[test]
    fn parses_single_element_array() {
        assert_eq!(
            parse_str("[a]").unwrap().root,
            AstNode::Arr(vec![AstNode::Word(char_string!("a"))])
        )
    }

    #[test]
    fn parses_double_element_array() {
        assert_eq!(
            parse_str("[a, b]").unwrap().root,
            AstNode::Arr(vec![
                AstNode::Word(char_string!("a")),
                AstNode::Word(char_string!("b"))
            ])
        )
    }

    #[test]
    fn parses_triple_element_array() {
        assert_eq!(
            parse_str("[a, b, c]").unwrap().root,
            AstNode::Arr(vec![
                AstNode::Word(char_string!("a")),
                AstNode::Word(char_string!("b")),
                AstNode::Word(char_string!("c"))
            ])
        )
    }

    #[test]
    fn parses_not_triple_element_array() {
        assert_eq!(
            parse_str("![a, b, c]").unwrap().root,
            AstNode::Not(Box::new(AstNode::Arr(vec![
                AstNode::Word(char_string!("a")),
                AstNode::Word(char_string!("b")),
                AstNode::Word(char_string!("c"))
            ])))
        )
    }
}
