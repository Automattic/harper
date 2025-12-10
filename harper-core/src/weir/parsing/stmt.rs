use crate::{CharString, CharStringExt, Punctuation, Token, TokenKind, TokenStringExt};

use super::expr::parse_seq;
use super::{Ast, AstExprNode, AstStmtNode, Error, FoundNode, lex, optimize};

pub fn parse_str(weir_code: &str, use_optimizer: bool) -> Result<Ast, Error> {
    let chars: CharString = weir_code.chars().collect();
    let tokens = lex(&chars);

    let mut stmts = parse_stmt_list(&tokens, &chars)?;

    if use_optimizer {
        while optimize(&mut stmts) {}
    }

    Ok(Ast { stmts })
}

fn parse_stmt_list(tokens: &[Token], source: &[char]) -> Result<Vec<AstStmtNode>, Error> {
    let mut list = Vec::new();

    let mut cursor = 0;
    while let Some(remainder) = tokens.get(cursor..)
        && !remainder.is_empty()
    {
        let res = parse_stmt(remainder, source)?;
        if let Some(node) = res.node {
            list.push(node);
        }
        cursor += res.next_idx;
    }
    Ok(list)
}

fn parse_stmt(tokens: &[Token], source: &[char]) -> Result<FoundNode<Option<AstStmtNode>>, Error> {
    let mut cursor = 0;

    // Skip whitespace at the beginning.
    while matches!(
        tokens.get(cursor).map(|t| &t.kind),
        Some(&TokenKind::Space(..) | &TokenKind::Newline(..))
    ) {
        cursor += 1;
    }

    let end = tokens
        .iter()
        .enumerate()
        .skip(cursor)
        .find_map(|(i, t)| t.kind.is_newline().then(|| i))
        .unwrap_or(tokens.len());

    let Some(key_token) = tokens.get(cursor) else {
        return Ok(FoundNode {
            node: None,
            next_idx: cursor + 1,
        });
    };

    match key_token.kind {
        TokenKind::Punctuation(Punctuation::Hash) => {
            let comment = tokens[cursor..end]
                .span()
                .unwrap()
                .get_content_string(source);
            Ok(FoundNode::new(Some(AstStmtNode::Comment(comment)), end + 2))
        }
        TokenKind::Word(_) => {
            let word_literal = key_token.span.get_content(source);

            match word_literal {
                ['d', 'e', 'c', 'l', 'a', 'r', 'e'] => Ok(FoundNode::new(
                    Some(AstStmtNode::create_declare_variable(
                        tokens[cursor + 2].span.get_content_string(source),
                        tokens[cursor + 4..end]
                            .span()
                            .unwrap()
                            .get_content_string(source),
                    )),
                    end + 1,
                )),
                ['s', 'e', 't'] => Ok(FoundNode::new(
                    Some(AstStmtNode::create_set_expr(
                        tokens[cursor + 2].span.get_content_string(source),
                        AstExprNode::Seq(parse_seq(&tokens[cursor + 4..end], source)?),
                    )),
                    end + 1,
                )),
                ['t', 'e', 's', 't'] => {
                    let case = parse_quoted_string(&tokens[cursor + 1..], source)?;
                    cursor += 1 + case.next_idx;
                    let sol = parse_quoted_string(&tokens[cursor + 1..], source)?;
                    cursor += 1 + sol.next_idx;

                    if cursor != end {
                        return Err(Error::UnexpectedToken(
                            tokens[cursor].span.get_content_string(source),
                        ));
                    }

                    Ok(FoundNode::new(
                        Some(AstStmtNode::create_test(case.node, sol.node)),
                        end + 1,
                    ))
                }
                _ => Err(Error::UnexpectedToken(word_literal.to_string())),
            }
        }
        _ => Err(Error::UnsupportedToken(
            key_token.span.get_content_string(source),
        )),
    }
}

fn parse_quoted_string(tokens: &[Token], source: &[char]) -> Result<FoundNode<String>, Error> {
    let mut cursor = 0;

    // Skip whitespace at the beginning.
    while matches!(
        tokens.get(cursor).map(|t| &t.kind),
        Some(&TokenKind::Space(..))
    ) {
        cursor += 1;
    }
    let quote_tok = tokens.get(cursor).ok_or(Error::EndOfInput)?;
    if !quote_tok.kind.is_quote() {
        return Err(Error::UnexpectedToken(
            quote_tok.span.get_content_string(source),
        ));
    }

    let end = tokens
        .iter()
        .enumerate()
        .skip(cursor + 1)
        .find_map(|(i, v)| v.kind.is_quote().then(|| i))
        .ok_or(Error::EndOfInput)?;

    dbg!(cursor, end);

    Ok(FoundNode {
        node: tokens[cursor + 1..end]
            .span()
            .unwrap_or_default()
            .get_content_string(source),
        next_idx: end + 1,
    })
}

#[cfg(test)]
mod tests {
    use crate::char_string::char_string;

    use super::{AstExprNode, AstStmtNode, parse_str};

    #[test]
    fn parses_single_var_stmt() {
        let ast = parse_str("declare test to be this", true).unwrap();

        assert_eq!(
            ast.stmts,
            vec![AstStmtNode::create_declare_variable("test", "to be this")]
        );
        assert_eq!(ast.get_variable_value("test"), Some("to be this"));
    }

    #[test]
    fn parses_single_expr_stmt() {
        assert_eq!(
            parse_str("set main word", true).unwrap().stmts,
            vec![AstStmtNode::create_set_expr(
                "main",
                AstExprNode::Word(char_string!("word"))
            )]
        )
    }

    #[test]
    fn parses_single_comment_stmt() {
        assert_eq!(
            parse_str("# this is a comment", true).unwrap().stmts,
            vec![AstStmtNode::Comment("# this is a comment".to_string())]
        )
    }

    #[test]
    fn parses_single_comment_stmt_with_space_prefix() {
        assert_eq!(
            parse_str("    # this is a comment", true).unwrap().stmts,
            vec![AstStmtNode::Comment("# this is a comment".to_string())]
        )
    }

    #[test]
    fn parses_tests() {
        assert_eq!(
            parse_str("test \"this is the case\" \"this is the solution\"", true)
                .unwrap()
                .stmts,
            vec![AstStmtNode::create_test(
                "this is the case",
                "this is the solution"
            )]
        )
    }

    #[test]
    fn parses_comment_expr_var_together() {
        let ast = parse_str(
            "declare test to be this\nset main word\n# this is a comment",
            true,
        )
        .unwrap();

        assert_eq!(
            ast.stmts,
            vec![
                AstStmtNode::create_declare_variable("test", "to be this"),
                AstStmtNode::create_set_expr("main", AstExprNode::Word(char_string!("word"))),
                AstStmtNode::Comment("# this is a comment".to_string())
            ]
        );

        assert_eq!(ast.get_variable_value("test"), Some("to be this"));
        assert_eq!(
            ast.get_expr("main"),
            Some(&AstExprNode::Word(char_string!("word")))
        );
    }
}
