use crate::{CharString, Punctuation, Token, TokenKind, TokenStringExt};

use super::expr::parse_seq;
use super::{Ast, AstExprNode, AstStmtNode, Error, FoundNode, lex, optimize};

pub fn parse_str(nesl: &str, use_optimizer: bool) -> Result<Ast, Error> {
    let chars: CharString = nesl.chars().collect();
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
        list.push(res.node);
        cursor += res.next_idx;
    }
    Ok(list)
}

fn parse_stmt(tokens: &[Token], source: &[char]) -> Result<FoundNode<AstStmtNode>, Error> {
    let end = tokens
        .iter()
        .position(|t| t.kind.is_newline())
        .unwrap_or(tokens.len());

    let mut cursor = 0;

    // Skip whitespace at the beginning.
    while matches!(
        tokens.get(cursor).map(|t| &t.kind),
        Some(&TokenKind::Space(..))
    ) {
        cursor += 1;
    }

    let key_token = tokens.get(cursor).ok_or(Error::EndOfInput)?;
    match key_token.kind {
        TokenKind::Punctuation(Punctuation::Hash) => {
            let comment = tokens[cursor..end]
                .span()
                .unwrap()
                .get_content_string(source);
            Ok(FoundNode::new(AstStmtNode::Comment(comment), end + 1))
        }
        TokenKind::Word(_) => {
            let word_literal = key_token.span.get_content(source);

            match word_literal {
                ['s', 'e', 't'] => Ok(FoundNode::new(
                    AstStmtNode::SetVariable {
                        name: tokens[cursor + 2].span.get_content_string(source),
                        value: tokens[cursor + 4..end]
                            .span()
                            .unwrap()
                            .get_content_string(source),
                    },
                    end + 1,
                )),
                ['p', 'r', 'o', 'd', 'u', 'c', 'e'] => Ok(FoundNode::new(
                    AstStmtNode::ProduceExpr(AstExprNode::Seq(parse_seq(
                        &tokens[cursor + 2..end],
                        source,
                    )?)),
                    end + 1,
                )),
                _ => Err(Error::UnexpectedKeyword),
            }
        }
        _ => Err(Error::UnsupportedToken),
    }
}

#[cfg(test)]
mod tests {
    use crate::char_string::char_string;
    use crate::nesl::ast::AstExprNode;

    use super::{AstStmtNode, parse_str};

    #[test]
    fn parses_single_var_stmt() {
        let ast = parse_str("set test to be this", true).unwrap();

        assert_eq!(
            ast.stmts,
            vec![AstStmtNode::create_set_variable("test", "to be this")]
        );
        assert_eq!(ast.get_variable_value("test"), Some("to be this"));
    }

    #[test]
    fn parses_single_expr_stmt() {
        assert_eq!(
            parse_str("produce word", true).unwrap().stmts,
            vec![AstStmtNode::ProduceExpr(AstExprNode::Word(char_string!(
                "word"
            )))]
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
    fn parses_comment_expr_var_together() {
        let ast = parse_str(
            "set test to be this\nproduce word\n# this is a comment",
            true,
        )
        .unwrap();

        assert_eq!(
            ast.stmts,
            vec![
                AstStmtNode::create_set_variable("test", "to be this"),
                AstStmtNode::ProduceExpr(AstExprNode::Word(char_string!("word"))),
                AstStmtNode::Comment("# this is a comment".to_string())
            ]
        );

        assert_eq!(ast.get_variable_value("test"), Some("to be this"));
    }
}
