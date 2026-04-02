use std::str::FromStr;

use harper_brill::UPOS;

use crate::{CharString, Currency, Punctuation, Token, TokenKind};

use super::{
    AstExprNode, Error, FoundNode, lex, locate_matching_brace, optimize_expr, parse_collection,
};

/// Parse a raw expression string.
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

/// Parse a sequence of expressions, one after the other.
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

/// Parse an individual expression.
fn parse_single_expr(tokens: &[Token], source: &[char]) -> Result<FoundNode<AstExprNode>, Error> {
    let cursor = 0;

    let tok = tokens.get(cursor).ok_or(Error::EndOfInput)?;

    match tok.kind {
        TokenKind::Space(_) => Ok(FoundNode::new(AstExprNode::Whitespace, 1)),
        // The expr ref notation
        TokenKind::Punctuation(Punctuation::At) => {
            let name_tok = tokens.get(1).ok_or(Error::EndOfInput)?;
            let name = name_tok.span.get_content(source);
            Ok(FoundNode::new(AstExprNode::ExprRef(name.into()), 2))
        }
        // The derivation notation.
        TokenKind::Punctuation(Punctuation::Currency(Currency::Dollar)) => {
            let word_tok = tokens.get(1).ok_or(Error::EndOfInput)?;

            let word = word_tok.span.get_content(source);
            Ok(FoundNode::new(AstExprNode::DerivativeOf(word.into()), 2))
        }
        TokenKind::Punctuation(Punctuation::Star) => {
            Ok(FoundNode::new(AstExprNode::Anything, 1))
        }
        TokenKind::Word(_) => {
            let text = tok.span.get_content_string(source);

            if let Ok(upos) = UPOS::from_str(&text){
                Ok(FoundNode::new(
                    AstExprNode::UPOSSet(vec![upos]),
                1,
                ))
            }else{
                let node = match text.as_str() {
                    "PROG" => AstExprNode::Progressive,
                    _ => AstExprNode::Word(text.chars().collect()),
                };

                Ok(FoundNode::new(
                        node,
                1,
                ))
            }
        }
        ,
        // The sequence notation. Useful for representing strings.
        TokenKind::Punctuation(Punctuation::OpenRound) => {
            let close_idx =
                locate_matching_brace(tokens, TokenKind::is_open_round, TokenKind::is_close_round)
                    .ok_or(Error::UnmatchedBrace)?;
            let child = parse_seq(&tokens[1..close_idx], source)?;
            Ok(FoundNode::new(AstExprNode::Seq(child), close_idx + 1))
        }
        // The _not_ or _unless_ notation
        TokenKind::Punctuation(Punctuation::Bang) => {
            let res = parse_single_expr(&tokens[1..], source)?;

            Ok(FoundNode::new(
                AstExprNode::Not(Box::new(res.node)),
                res.next_idx + 1,
            ))
        }
        // The array notation
        TokenKind::Punctuation(Punctuation::OpenSquare) => {
            let close_idx = locate_matching_brace(
                tokens,
                TokenKind::is_open_square,
                TokenKind::is_close_square,
            )
            .ok_or(Error::UnmatchedBrace)?;

            let children = parse_collection(&tokens[1..close_idx], source, parse_single_expr)?;

            Ok(FoundNode::new(AstExprNode::Arr(children), close_idx + 1))
        }
        // The filter notation
        TokenKind::Punctuation(Punctuation::LessThan) => {
            let close_idx =
                locate_matching_brace(tokens, TokenKind::is_less_than, TokenKind::is_greater_than)
                    .ok_or(Error::UnmatchedBrace)?;

            let children = parse_collection(&tokens[1..close_idx], source, parse_single_expr)?;

            Ok(FoundNode::new(AstExprNode::Filter(children), close_idx + 1))
        }

        TokenKind::Punctuation(p) => Ok(FoundNode::new(AstExprNode::Punctuation(p), 1)),
        _ => Err(Error::UnsupportedToken(tok.span.get_content_string(source))),
    }
}

#[cfg(test)]
mod tests {
    use harper_brill::UPOS;

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
    fn parses_space_dash() {
        assert_eq!(
            parse_expr_str("[( ), (-)]", true).unwrap(),
            AstExprNode::Arr(vec![
                AstExprNode::Whitespace,
                AstExprNode::Punctuation(Punctuation::Hyphen),
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

    #[test]
    fn parses_upos() {
        assert_eq!(
            parse_expr_str("PROPN NOUN VERB", false).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::UPOSSet(vec![UPOS::PROPN]),
                AstExprNode::Whitespace,
                AstExprNode::UPOSSet(vec![UPOS::NOUN]),
                AstExprNode::Whitespace,
                AstExprNode::UPOSSet(vec![UPOS::VERB]),
            ])
        )
    }

    #[test]
    fn optimizes_upos_set() {
        assert_eq!(
            parse_expr_str("[PROPN, NOUN, VERB]", true).unwrap(),
            AstExprNode::UPOSSet(vec![UPOS::PROPN, UPOS::NOUN, UPOS::VERB]),
        )
    }

    #[test]
    fn parses_prog() {
        assert_eq!(
            parse_expr_str("PROG", true).unwrap(),
            AstExprNode::Progressive
        )
    }

    #[test]
    fn parses_expr_ref() {
        assert_eq!(
            parse_expr_str("@test", true).unwrap(),
            AstExprNode::ExprRef(char_string!("test"))
        )
    }

    #[test]
    fn parses_expr_ref_array() {
        assert_eq!(
            parse_expr_str("[@a, @b, @c]", true).unwrap(),
            AstExprNode::Arr(vec![
                AstExprNode::ExprRef(char_string!("a")),
                AstExprNode::ExprRef(char_string!("b")),
                AstExprNode::ExprRef(char_string!("c"))
            ])
        )
    }

    #[test]
    fn parses_anything() {
        assert_eq!(parse_expr_str("*", true).unwrap(), AstExprNode::Anything)
    }

    #[test]
    fn parses_anything_seq() {
        assert_eq!(
            parse_expr_str("* * *", true).unwrap(),
            AstExprNode::Seq(vec![
                AstExprNode::Anything,
                AstExprNode::Whitespace,
                AstExprNode::Anything,
                AstExprNode::Whitespace,
                AstExprNode::Anything,
            ])
        )
    }
}
