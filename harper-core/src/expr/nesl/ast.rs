use is_macro::Is;

use crate::expr::{Expr, FirstMatchOf, SequenceExpr, UnlessStep};
use crate::patterns::{WhitespacePattern, Word};
use crate::{CharString, Token};

#[derive(Debug, Clone, Is, Eq, PartialEq)]
pub enum AstNode {
    Whitespace,
    Word(CharString),
    Not(Box<AstNode>),
    Seq(Vec<AstNode>),
    Arr(Vec<AstNode>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ast {
    pub root: AstNode,
}

impl AstNode {
    /// Create an expression that fulfills the pattern matching contract defined by this tree.
    pub fn to_expr(&self) -> Box<dyn Expr> {
        match self {
            AstNode::Whitespace => Box::new(WhitespacePattern),
            AstNode::Word(word) => Box::new(Word::from_chars(&word)),
            AstNode::Not(ast_node) => Box::new(UnlessStep::new(
                ast_node.to_expr(),
                |_tok: &Token, _: &[char]| true,
            )),
            AstNode::Seq(children) => {
                let mut expr = SequenceExpr::default();

                for node in children {
                    expr = expr.then_boxed(node.to_expr());
                }

                Box::new(expr)
            }
            AstNode::Arr(children) => {
                let mut expr = FirstMatchOf::default();

                for node in children {
                    expr.add_boxed(node.to_expr());
                }

                Box::new(expr)
            }
        }
    }
}
