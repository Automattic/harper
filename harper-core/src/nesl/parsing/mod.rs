mod expr;
mod stmt;

use super::Error;
use ast::{Ast, AstExprNode, AstStmtNode};

pub use expr::parse_expr_str;
pub use stmt::parse_str;

use crate::lexing::{FoundToken, lex_nesl_token};
use crate::{Span, Token};

use super::{
    ast,
    optimize::{optimize, optimize_expr},
};

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

#[derive(Debug)]
struct FoundNode<T> {
    /// The parsed node found.
    node: T,
    /// The next token to be read.
    next_idx: usize,
}

impl<T> FoundNode<T> {
    pub fn new(node: T, next_idx: usize) -> Self {
        Self { node, next_idx }
    }
}
