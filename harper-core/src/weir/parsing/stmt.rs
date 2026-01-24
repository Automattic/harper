use std::num::NonZeroUsize;
use std::sync::{Arc, LazyLock, RwLock};

use lru::LruCache;

use super::ast::AstVariable;
use super::inc_by_spaces;
use crate::{CharStringExt, Punctuation, Token, TokenKind, TokenStringExt};

use super::expr::parse_seq;
use super::{
    Ast, AstExprNode, AstStmtNode, Error, FoundNode, expected_space, inc_by_whitespace, lex,
    locate_matching_brace, optimize, parse_collection,
};

/// Parse the provided string using caching to speed up repeated requests.
///
/// Will return the cached AST if the provided code has previously been cached. Otherwise, it will
/// parse the code and cache the resulting AST.
pub fn parse_str(weir_code: &str, use_optimizer: bool) -> Result<Arc<Ast>, Error> {
    type ParseStrParams = (String, bool);

    static PARSE_CACHE: LazyLock<
        RwLock<LruCache<ParseStrParams, Arc<Ast>, hashbrown::DefaultHashBuilder>>,
    > = LazyLock::new(|| RwLock::new(LruCache::new(NonZeroUsize::new(10000).unwrap())));

    Ok(
        if let Some(cached) = PARSE_CACHE
            .read()
            .unwrap()
            .peek(&(weir_code.to_owned(), use_optimizer))
        {
            cached.clone()
        } else {
            let chars: Vec<char> = weir_code.chars().collect();
            let tokens = lex(&chars);

            let mut stmts = parse_stmt_list(&tokens, &chars)?;

            if use_optimizer {
                while optimize(&mut stmts) {}
            }

            let ast = Arc::new(Ast { stmts });

            PARSE_CACHE
                .write()
                .unwrap()
                .put((weir_code.to_owned(), use_optimizer), ast.clone());

            ast
        },
    )
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
    inc_by_whitespace(&mut cursor, tokens);

    let end = tokens
        .iter()
        .enumerate()
        .skip(cursor)
        .find_map(|(i, t)| t.kind.is_newline().then_some(i))
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
            Ok(FoundNode::new(Some(AstStmtNode::Comment(comment)), end + 1))
        }
        TokenKind::Word(_) => {
            let word_literal = key_token.span.get_content(source);

            match word_literal {
                ['l', 'e', 't'] => {
                    expected_space(cursor + 1, tokens, source)?;
                    let name = tokens[cursor + 2].span.get_content_string(source);
                    expected_space(cursor + 3, tokens, source)?;

                    let str_res = parse_quoted_string(&tokens[cursor + 4..end], source);

                    if let Ok(str_contents) = str_res {
                        if tokens[str_contents.next_idx + cursor + 4..end]
                            .iter()
                            .any(|t| !t.kind.is_space())
                        {
                            return Err(Error::UnexpectedToken(
                                tokens[str_contents.next_idx + cursor + 4]
                                    .span
                                    .get_content_string(source),
                            ));
                        }

                        Ok(FoundNode::new(
                            Some(AstStmtNode::create_declare_variable(
                                name,
                                AstVariable::String(str_contents.node),
                            )),
                            end + 1,
                        ))
                    } else {
                        let open_brac_tok = &tokens[cursor + 4];
                        if !open_brac_tok.kind.is_open_square() {
                            return Err(Error::UnexpectedToken(
                                open_brac_tok.span.get_content_string(source),
                            ));
                        }

                        let matching = locate_matching_brace(
                            &tokens[cursor + 4..end],
                            TokenKind::is_open_square,
                            TokenKind::is_close_square,
                        )
                        .ok_or(Error::UnmatchedBrace)?
                            + cursor
                            + 4;

                        let elements = parse_collection(
                            &tokens[cursor + 5..matching],
                            source,
                            parse_quoted_string,
                        )?;

                        Ok(FoundNode::new(
                            Some(AstStmtNode::create_declare_variable(
                                name,
                                AstVariable::Array(
                                    elements.into_iter().map(AstVariable::String).collect(),
                                ),
                            )),
                            end + 1,
                        ))
                    }
                }
                ['e', 'x', 'p', 'r'] => {
                    expected_space(cursor + 1, tokens, source)?;

                    Ok(FoundNode::new(
                        Some(AstStmtNode::create_set_expr(
                            tokens[cursor + 2].span.get_content_string(source),
                            AstExprNode::Seq(parse_seq(
                                &tokens[(cursor + 4).min(end)..end],
                                source,
                            )?),
                        )),
                        end + 1,
                    ))
                }
                ['t', 'e', 's', 't'] => {
                    let case = parse_quoted_string(&tokens[cursor + 1..], source)?;
                    cursor += 1 + case.next_idx;

                    expected_space(cursor, tokens, source)?;

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
    fn is_escaped(pos: usize, source: &[char]) -> bool {
        if pos == 0 {
            return false;
        }
        let mut i = pos;
        let mut n = 0;
        while i > 0 && source[i - 1] == '\\' {
            n += 1;
            i -= 1;
        }
        n % 2 == 1
    }

    let mut cursor = 0;

    inc_by_spaces(&mut cursor, tokens);

    let quote_tok = tokens.get(cursor).ok_or(Error::EndOfInput)?;
    if !quote_tok.kind.is_quote() {
        return Err(Error::UnexpectedToken(
            quote_tok.span.get_content_string(source),
        ));
    }

    let mut end = None;
    for (i, tok) in tokens.iter().enumerate().skip(cursor + 1) {
        if tok.kind.is_quote() {
            let qpos = tok.span.start;
            if !is_escaped(qpos, source) {
                end = Some(i);
                break;
            }
        }
    }

    let end = end.ok_or(Error::EndOfInput)?;

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
    use quickcheck_macros::quickcheck;

    use crate::char_string::char_string;

    use super::{AstExprNode, AstStmtNode, AstVariable, parse_str};

    #[test]
    fn parses_single_var_stmt() {
        let ast = parse_str("let test \"to be this\"", true).unwrap();

        assert_eq!(
            ast.stmts,
            vec![AstStmtNode::create_declare_variable(
                "test",
                AstVariable::create_string("to be this")
            ),]
        );
        assert_eq!(
            ast.get_variable_value("test"),
            Some(&AstVariable::create_string("to be this"))
        );
    }

    #[test]
    fn parses_single_var_stmt_with_lots_of_space() {
        let ast = parse_str("let            test \"to be this\"", true).unwrap();

        assert_eq!(
            ast.stmts,
            vec![AstStmtNode::create_declare_variable(
                "test",
                AstVariable::create_string("to be this")
            ),]
        );
        assert_eq!(
            ast.get_variable_value("test"),
            Some(&AstVariable::create_string("to be this"))
        );
    }

    #[test]
    fn parses_single_var_stmt_array() {
        let ast = parse_str("let test [\"to be this\", \"and this\"]", true).unwrap();

        let correct_var_val = AstVariable::Array(vec![
            AstVariable::create_string("to be this"),
            AstVariable::create_string("and this"),
        ]);

        assert_eq!(ast.get_variable_value("test"), Some(&correct_var_val));
        assert_eq!(
            ast.stmts,
            vec![AstStmtNode::create_declare_variable(
                "test",
                correct_var_val
            ),]
        );
    }

    #[test]
    fn parses_single_expr_stmt() {
        assert_eq!(
            parse_str("expr main word", true).unwrap().stmts,
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
    fn parses_multiple_spaces_in_tests() {
        assert_eq!(
            parse_str(
                "test \"this is the case\"           \"this is the solution\"",
                true
            )
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
            "let test \"to be this\"\nexpr main word\n# this is a comment",
            true,
        )
        .unwrap();

        assert_eq!(
            ast.stmts,
            vec![
                AstStmtNode::create_declare_variable(
                    "test",
                    AstVariable::create_string("to be this")
                ),
                AstStmtNode::create_set_expr("main", AstExprNode::Word(char_string!("word"))),
                AstStmtNode::Comment("# this is a comment".to_string())
            ]
        );

        assert_eq!(
            ast.get_variable_value("test"),
            Some(&AstVariable::create_string("to be this"))
        );
        assert_eq!(
            ast.get_expr("main"),
            Some(&AstExprNode::Word(char_string!("word")))
        );
    }

    #[test]
    fn parses_comment_in_middle() {
        let ast = parse_str(
            "let test \"to be this\"\n# this is a comment\nexpr main word",
            true,
        )
        .unwrap();

        assert_eq!(
            ast.stmts,
            vec![
                AstStmtNode::create_declare_variable(
                    "test",
                    AstVariable::create_string("to be this")
                ),
                AstStmtNode::Comment("# this is a comment".to_string()),
                AstStmtNode::create_set_expr("main", AstExprNode::Word(char_string!("word")))
            ]
        );

        assert_eq!(
            ast.get_variable_value("test"),
            Some(&AstVariable::create_string("to be this"))
        );
        assert_eq!(
            ast.get_expr("main"),
            Some(&AstExprNode::Word(char_string!("word")))
        );
    }

    #[test]
    #[should_panic]
    fn catches_non_whitespace_after_expr() {
        parse_str("expr+this is a test", false).unwrap();
    }

    #[test]
    #[should_panic]
    fn catches_non_whitespace_after_test() {
        parse_str("test+\"\" \"\"", false).unwrap();
    }

    #[test]
    #[should_panic]
    fn catches_non_whitespace_between_test() {
        parse_str("test \"\"+\"\"", false).unwrap();
    }

    #[test]
    #[should_panic]
    fn catches_non_whitespace_after_let() {
        parse_str("let+var \"\"", false).unwrap();
    }

    #[test]
    #[should_panic]
    fn catches_non_whitespace_after_let_var() {
        parse_str("let var+\"\"", false).unwrap();
    }

    #[quickcheck]
    fn catches_anything_after_test(a: String) {
        if !a.is_empty() && !a.starts_with('\n') {
            let code = format!("test \"\" \"\"{a}");
            assert!(parse_str(code.as_str(), false).is_err())
        }
    }
}
