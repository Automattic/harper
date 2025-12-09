use is_macro::Is;

use crate::expr::{Expr, FirstMatchOf, SequenceExpr, UnlessStep};
use crate::patterns::{WhitespacePattern, Word};
use crate::{CharString, Punctuation, Token};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ast {
    pub stmts: Vec<AstStmtNode>,
}

impl Ast {
    pub fn new(stmts: Vec<AstStmtNode>) -> Self {
        Self { stmts }
    }

    pub fn get_variable_value(&self, var_name: &str) -> Option<&'_ str> {
        for stmt in self.stmts.iter().rev() {
            if let AstStmtNode::DeclareVariable { name, value } = stmt
                && name == var_name
            {
                return Some(value.as_str());
            }
        }
        None
    }

    pub fn get_expr(&self, expr_name: &str) -> Option<&'_ AstExprNode> {
        for stmt in self.stmts.iter().rev() {
            if let AstStmtNode::SetExpr { name, value } = stmt
                && name == expr_name
            {
                return Some(value);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Is, Eq, PartialEq)]
pub enum AstExprNode {
    Whitespace,
    Word(CharString),
    Punctuation(Punctuation),
    Not(Box<AstExprNode>),
    Seq(Vec<AstExprNode>),
    Arr(Vec<AstExprNode>),
}

impl AstExprNode {
    /// Create an expression that fulfills the pattern matching contract defined by this tree.
    pub fn to_expr(&self) -> Box<dyn Expr> {
        match self {
            AstExprNode::Whitespace => Box::new(WhitespacePattern),
            AstExprNode::Word(word) => Box::new(Word::from_chars(word)),
            AstExprNode::Not(ast_node) => Box::new(UnlessStep::new(
                ast_node.to_expr(),
                |_tok: &Token, _: &[char]| true,
            )),
            AstExprNode::Seq(children) => {
                let mut expr = SequenceExpr::default();

                for node in children {
                    expr = expr.then_boxed(node.to_expr());
                }

                Box::new(expr)
            }
            AstExprNode::Arr(children) => {
                let mut expr = FirstMatchOf::default();

                for node in children {
                    expr.add_boxed(node.to_expr());
                }

                Box::new(expr)
            }
            AstExprNode::Punctuation(punct) => {
                let punct = *punct;

                Box::new(move |tok: &Token, _: &[char]| {
                    tok.kind.as_punctuation().is_some_and(|p| *p == punct)
                })
            }
        }
    }
}

#[derive(Debug, Clone, Is, Eq, PartialEq)]
pub enum AstStmtNode {
    DeclareVariable { name: String, value: String },
    SetExpr { name: String, value: AstExprNode },
    Comment(String),
}

impl AstStmtNode {
    pub fn create_declare_variable(name: impl ToString, value: impl ToString) -> Self {
        Self::DeclareVariable {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    pub fn create_set_expr(name: impl ToString, value: AstExprNode) -> Self {
        Self::SetExpr {
            name: name.to_string(),
            value,
        }
    }
}
