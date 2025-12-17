mod expr;
mod stmt;

use super::Error;
use ast::{Ast, AstExprNode, AstStmtNode};

pub use expr::parse_expr_str;
pub use stmt::parse_str;

use crate::lexing::{FoundToken, lex_weir_token};
use crate::{Span, Token, TokenKind};

use super::{
    ast,
    optimize::{optimize, optimize_expr},
};

/// Lex the entirety of a Weir document.
fn lex(source: &[char]) -> Vec<Token> {
    let mut cursor = 0;

    let mut tokens = Vec::new();

    loop {
        if cursor >= source.len() {
            return tokens;
        }

        if let Some(FoundToken { token, next_index }) = lex_weir_token(&source[cursor..]) {
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

/// A utility for parsing a collection of items, separated by commas.
/// Requires a parser used for parsing individual elements.
fn parse_collection<T>(
    tokens: &[Token],
    source: &[char],
    el_parser: impl Fn(&[Token], &[char]) -> Result<FoundNode<T>, Error>,
) -> Result<Vec<T>, Error> {
    let mut children = Vec::new();

    let mut cursor = 0;

    while cursor < tokens.len() {
        while tokens[cursor].kind.is_space() {
            cursor += 1;
        }

        let new_child = el_parser(&tokens[cursor..], source)?;
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

    inc_by_spaces(&mut cursor, tokens);

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

/// Increments the cursor until it is no longer over a space.
fn inc_by_spaces(cursor: &mut usize, tokens: &[Token]) {
    // Skip whitespace at the beginning.
    while matches!(
        tokens.get(*cursor).map(|t| &t.kind),
        Some(&TokenKind::Space(..))
    ) {
        *cursor += 1;
    }
}

/// Increments the cursor until it is no longer over whitespace.
fn inc_by_whitespace(cursor: &mut usize, tokens: &[Token]) {
    // Skip whitespace at the beginning.
    while tokens
        .get(*cursor)
        .map(|t| &t.kind)
        .is_some_and(|t| t.is_whitespace())
    {
        *cursor += 1;
    }
}

/// Asserts that a space is expected in the location of the cursor.
/// Returns the proper arrow type that can be handled with the `?` syntax.
fn expected_space(cursor: usize, tokens: &[Token], source: &[char]) -> Result<(), Error> {
    let expected_space = &tokens[cursor];

    if !expected_space.kind.is_space() {
        return Err(Error::UnexpectedToken(
            expected_space.span.get_content_string(source),
        ));
    }

    Ok(())
}
