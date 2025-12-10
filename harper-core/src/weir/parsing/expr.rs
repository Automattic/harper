use crate::lexing::{FoundToken, lex_weir_token};
use crate::{CharString, Currency, Punctuation, Token, TokenKind, TokenStringExt};

use super::{AstExprNode, Error, FoundNode, lex, optimize_expr};

pub fn parse_expr_str(weir_code: &str, use_optimizer: bool) -> Result<AstExprNode, Error> {
    let chars: CharString = weir_code.chars().collect();
    let tokens = lex(&chars);

    let seq = parse_seq(&tokens, &chars)?;
    let mut root = AstExprNode::Seq(seq);

    if use_optimizer {
        while optimize_expr(&mut root) {}
    }

    Ok(root)
}

pub fn parse_seq(tokens: &[Token], source: &[char]) -> Result<Vec<AstExprNode>, Error> {
    let mut seq = Vec::new();

    let mut cursor = 0;
    while let Some(remainder) = tokens.get(cursor..)
        && !remainder.is_empty()
    {
        let res = parse_single_expr(remainder, source)?;
        seq.push(res.node);
        cursor += res.next_idx;
    }

    Ok(seq)
}

fn parse_single_expr(tokens: &[Token], source: &[char]) -> Result<FoundNode<AstExprNode>, Error> {
    let tok = tokens.first().ok_or(Error::EndOfInput)?;

    match tok.kind {
        TokenKind::Space(_) => Ok(FoundNode::new(AstExprNode::Whitespace, 1)),
        TokenKind::Punctuation(Punctuation::Currency(Currency::Dollar)) => {
            let word_tok = tokens.get(1).ok_or(Error::EndOfInput)?;

            let word = word_tok.span.get_content(source);
            Ok(FoundNode::new(AstExprNode::DerivativeOf(word.into()), 2))
        }
        TokenKind::Word(_) => Ok(FoundNode::new(
            AstExprNode::Word(tok.span.get_content(source).into()),
            1,
        )),
        TokenKind::Punctuation(Punctuation::OpenRound) => {
            let close_idx =
                locate_matching_brace(tokens, TokenKind::is_open_round, TokenKind::is_close_round)
                    .ok_or(Error::UnmatchedBrace)?;
            let child = parse_seq(&tokens[1..close_idx], source)?;
            Ok(FoundNode::new(AstExprNode::Seq(child), close_idx + 1))
        }

        TokenKind::Punctuation(Punctuation::Bang) => {
            let res = parse_single_expr(&tokens[1..], source)?;

            Ok(FoundNode::new(
                AstExprNode::Not(Box::new(res.node)),
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

            let children = parse_collection(&tokens[1..close_idx], source)?;

            Ok(FoundNode::new(AstExprNode::Arr(children), close_idx + 1))
        }
        TokenKind::Punctuation(Punctuation::LessThan) => {
            let close_idx =
                locate_matching_brace(tokens, TokenKind::is_less_than, TokenKind::is_greater_than)
                    .ok_or(Error::UnmatchedBrace)?;

            let children = parse_collection(&tokens[1..close_idx], source)?;

            Ok(FoundNode::new(AstExprNode::Filter(children), close_idx + 1))
        }

        TokenKind::Punctuation(p) => Ok(FoundNode::new(AstExprNode::Punctuation(p), 1)),
        _ => Err(Error::UnsupportedToken(tok.span.get_content_string(source))),
    }
}

fn parse_collection(tokens: &[Token], source: &[char]) -> Result<Vec<AstExprNode>, Error> {
    let mut children = Vec::new();

    let mut cursor = 0;

    while cursor < tokens.len() {
        while tokens[cursor].kind.is_space() {
            cursor += 1;
        }

        let new_child = parse_single_expr(&tokens[cursor..], source)?;
        children.push(new_child.node);

        cursor += new_child.next_idx;

        while cursor != tokens.len() && tokens[cursor].kind.is_space() {
            cursor += 1;
        }

        if cursor != tokens.len() && !tokens[cursor].kind.is_comma() {
            return Err(Error::ExpectedComma);
        }

        cursor += 1;

        if cursor < tokens.len() && tokens[cursor].kind.is_space() {
            cursor += 1;
        }
    }

    Ok(children)
}

/// Locates the closing brace for the token at index zero.
fn locate_matching_brace(
    tokens: &[Token],
    is_open: impl Fn(&TokenKind) -> bool,
    is_close: impl Fn(&TokenKind) -> bool,
) -> Option<usize> {
    // Locate closing brace
    let mut visited_opens = 0;
    let mut cursor = 1;

    loop {
        let cur = tokens.get(cursor)?;

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
    use crate::Punctuation;
    use crate::char_string::char_string;

    use super::{AstExprNode, parse_expr_str};

    #[test]
    fn parses_whitespace() {
        assert_eq!(parse_expr_str(" ", true).unwrap(), AstExprNode::Whitespace)
    }

    #[test]
    fn parses_word() {
        assert_eq!(
            parse_expr_str("word", true).unwrap(),
            AstExprNode::Word(char_string!("word"))
        )
    }

    #[test]
    fn parses_word_space() {
        assert_eq!(
            parse_expr_str("word ", true).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::Word(char_string!("word")),
                AstExprNode::Whitespace
            ])
        )
    }

    #[test]
    fn parses_word_space_word() {
        assert_eq!(
            parse_expr_str("word word", true).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::Word(char_string!("word")),
                AstExprNode::Whitespace,
                AstExprNode::Word(char_string!("word")),
            ])
        )
    }

    #[test]
    fn parses_simple_seq() {
        assert_eq!(
            parse_expr_str("a (b c) d", false).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::Word(char_string!("a")),
                AstExprNode::Whitespace,
                AstExprNode::Seq(vec![
                    AstExprNode::Word(char_string!("b")),
                    AstExprNode::Whitespace,
                    AstExprNode::Word(char_string!("c")),
                ]),
                AstExprNode::Whitespace,
                AstExprNode::Word(char_string!("d")),
            ])
        )
    }

    #[test]
    fn parses_nested_seqs() {
        assert_eq!(
            parse_expr_str("a (b (c)) d", false).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::Word(char_string!("a")),
                AstExprNode::Whitespace,
                AstExprNode::Seq(vec![
                    AstExprNode::Word(char_string!("b")),
                    AstExprNode::Whitespace,
                    AstExprNode::Seq(vec![AstExprNode::Word(char_string!("c")),]),
                ]),
                AstExprNode::Whitespace,
                AstExprNode::Word(char_string!("d")),
            ])
        )
    }

    #[test]
    fn parses_paired_seqs() {
        assert_eq!(
            parse_expr_str("a (b) (c) d", false).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::Word(char_string!("a")),
                AstExprNode::Whitespace,
                AstExprNode::Seq(vec![AstExprNode::Word(char_string!("b")),]),
                AstExprNode::Whitespace,
                AstExprNode::Seq(vec![AstExprNode::Word(char_string!("c")),]),
                AstExprNode::Whitespace,
                AstExprNode::Word(char_string!("d")),
            ])
        )
    }

    #[test]
    fn parses_not() {
        assert_eq!(
            parse_expr_str("a !b c", false).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::Word(char_string!("a")),
                AstExprNode::Whitespace,
                AstExprNode::Not(Box::new(AstExprNode::Word(char_string!("b")))),
                AstExprNode::Whitespace,
                AstExprNode::Word(char_string!("c")),
            ])
        )
    }

    #[test]
    fn parses_not_seq() {
        assert_eq!(
            parse_expr_str("a !(b c) d", false).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::Word(char_string!("a")),
                AstExprNode::Whitespace,
                AstExprNode::Not(Box::new(AstExprNode::Seq(vec![
                    AstExprNode::Word(char_string!("b")),
                    AstExprNode::Whitespace,
                    AstExprNode::Word(char_string!("c")),
                ]),)),
                AstExprNode::Whitespace,
                AstExprNode::Word(char_string!("d")),
            ])
        )
    }

    #[test]
    fn parses_empty_array() {
        assert_eq!(
            parse_expr_str("[]", true).unwrap(),
            AstExprNode::Arr(vec![])
        )
    }

    #[test]
    fn parses_single_element_array() {
        assert_eq!(
            parse_expr_str("[a]", false).unwrap(),
            AstExprNode::Seq(vec![AstExprNode::Arr(vec![AstExprNode::Word(
                char_string!("a")
            )])])
        )
    }

    #[test]
    fn optimizer_deconstructs_single_element_array() {
        assert_eq!(
            parse_expr_str("[a]", true).unwrap(),
            AstExprNode::Word(char_string!("a"))
        )
    }

    #[test]
    fn optimizer_deconstructs_single_element_seq() {
        assert_eq!(
            parse_expr_str("(a)", true).unwrap(),
            AstExprNode::Word(char_string!("a"))
        )
    }

    #[test]
    fn parses_double_element_array() {
        assert_eq!(
            parse_expr_str("[a, b]", true).unwrap(),
            AstExprNode::Arr(vec![
                AstExprNode::Word(char_string!("a")),
                AstExprNode::Word(char_string!("b"))
            ])
        )
    }

    #[test]
    fn parses_triple_element_array() {
        assert_eq!(
            parse_expr_str("[a, b, c]", true).unwrap(),
            AstExprNode::Arr(vec![
                AstExprNode::Word(char_string!("a")),
                AstExprNode::Word(char_string!("b")),
                AstExprNode::Word(char_string!("c"))
            ])
        )
    }

    #[test]
    fn parses_not_triple_element_array() {
        assert_eq!(
            parse_expr_str("![a, b, c]", true).unwrap(),
            AstExprNode::Not(Box::new(AstExprNode::Arr(vec![
                AstExprNode::Word(char_string!("a")),
                AstExprNode::Word(char_string!("b")),
                AstExprNode::Word(char_string!("c"))
            ])))
        )
    }

    #[test]
    fn parses_triple_dot() {
        assert_eq!(
            parse_expr_str("...", false).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::Punctuation(Punctuation::Period),
                AstExprNode::Punctuation(Punctuation::Period),
                AstExprNode::Punctuation(Punctuation::Period),
            ])
        )
    }

    #[test]
    fn parses_space_comma() {
        assert_eq!(
            parse_expr_str("[( ), (,)]", true).unwrap(),
            AstExprNode::Arr(vec![
                AstExprNode::Whitespace,
                AstExprNode::Punctuation(Punctuation::Comma),
            ])
        )
    }

    #[test]
    fn parses_filter() {
        assert_eq!(
            parse_expr_str("<a, b, c>", true).unwrap(),
            AstExprNode::Filter(vec![
                AstExprNode::Word(char_string!("a")),
                AstExprNode::Word(char_string!("b")),
                AstExprNode::Word(char_string!("c")),
            ])
        )
    }

    #[test]
    fn parses_filter_with_space_prefixing_element() {
        assert_eq!(
            parse_expr_str("< a, b, c>", true).unwrap(),
            AstExprNode::Filter(vec![
                AstExprNode::Word(char_string!("a")),
                AstExprNode::Word(char_string!("b")),
                AstExprNode::Word(char_string!("c")),
            ])
        )
    }

    #[test]
    fn parses_filter_with_space_postfixing_element() {
        assert_eq!(
            parse_expr_str("<a, b, c >", true).unwrap(),
            AstExprNode::Filter(vec![
                AstExprNode::Word(char_string!("a")),
                AstExprNode::Word(char_string!("b")),
                AstExprNode::Word(char_string!("c")),
            ])
        )
    }

    #[test]
    fn parses_derivative() {
        assert_eq!(
            parse_expr_str("$word", true).unwrap(),
            AstExprNode::DerivativeOf(char_string!("word"))
        )
    }

    #[test]
    fn parses_derivative_seq() {
        assert_eq!(
            parse_expr_str("$a $b $c", true).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::DerivativeOf(char_string!("a")),
                AstExprNode::Whitespace,
                AstExprNode::DerivativeOf(char_string!("b")),
                AstExprNode::Whitespace,
                AstExprNode::DerivativeOf(char_string!("c")),
            ])
        )
    }

    #[test]
    fn parses_contraction() {
        assert_eq!(
            parse_expr_str("don't do this", true).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::Word(char_string!("don't")),
                AstExprNode::Whitespace,
                AstExprNode::Word(char_string!("do")),
                AstExprNode::Whitespace,
                AstExprNode::Word(char_string!("this")),
            ])
        )
    }
}
