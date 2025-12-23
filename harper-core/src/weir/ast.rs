use harper_brill::UPOS;
use is_macro::Is;
use itertools::Itertools;

use crate::expr::{Expr, Filter, FirstMatchOf, SequenceExpr, UnlessStep};
use crate::patterns::{AnyPattern, DerivedFrom, UPOSSet, WhitespacePattern, Word};
use crate::{CharString, Punctuation, Token, TokenKind};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ast {
    pub stmts: Vec<AstStmtNode>,
}

impl Ast {
    /// Construct a new abstract syntax tree from individual statements.
    pub fn new(stmts: Vec<AstStmtNode>) -> Self {
        Self { stmts }
    }

    /// Get the value of a variable from the last time it was set.
    pub fn get_variable_value(&self, var_name: &str) -> Option<&'_ AstVariable> {
        for stmt in self.stmts.iter().rev() {
            if let AstStmtNode::DeclareVariable { name, value } = stmt
                && name == var_name
            {
                return Some(value);
            }
        }
        None
    }

    /// Get the value of an expression from the last time it was set.
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

    /// Iterate through all unique variable values, from the last time they were set.
    pub fn iter_variable_values(&self) -> impl Iterator<Item = (&str, &AstVariable)> {
        self.stmts
            .iter()
            .rev()
            .filter_map(|n| match n {
                AstStmtNode::DeclareVariable { name, value } => Some((name.as_str(), value)),
                _ => None,
            })
            .unique_by(|(n, _)| *n)
    }

    /// Iterate through all the tests in the tree, starting with the one first declared in the
    /// tree.
    pub fn iter_tests(&self) -> impl Iterator<Item = (&str, &str)> {
        self.stmts.iter().filter_map(|stmt| {
            if let AstStmtNode::Test { expect, to_be } = stmt {
                Some((expect.as_str(), to_be.as_str()))
            } else {
                None
            }
        })
    }
}

/// A node that represents an expression that can be used to search through natural language.
#[derive(Debug, Clone, Is, Eq, PartialEq)]
pub enum AstExprNode {
    Whitespace,
    /// A progressive verb.
    Progressive,
    UPOSSet(Vec<UPOS>),
    Word(CharString),
    DerivativeOf(CharString),
    Punctuation(Punctuation),
    Not(Box<AstExprNode>),
    Seq(Vec<AstExprNode>),
    Arr(Vec<AstExprNode>),
    Filter(Vec<AstExprNode>),
    Anything,
}

impl AstExprNode {
    /// Create an actual expression that fulfills the pattern matching contract defined by this tree.
    pub fn to_expr(&self) -> Box<dyn Expr> {
        match self {
            AstExprNode::Anything => Box::new(AnyPattern),
            AstExprNode::Progressive => {
                Box::new(|tok: &Token, _: &[char]| tok.kind.is_verb_progressive_form())
            }
            AstExprNode::UPOSSet(upos) => Box::new(UPOSSet::new(&upos)),
            AstExprNode::Whitespace => Box::new(WhitespacePattern),
            AstExprNode::Word(word) => Box::new(Word::from_chars(word)),
            AstExprNode::DerivativeOf(word) => Box::new(DerivedFrom::new_from_chars(word)),
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
            AstExprNode::Filter(children) => {
                Box::new(Filter::new(children.iter().map(|n| n.to_expr()).collect()))
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

/// A variable defined by the `let` keyword.
#[derive(Debug, Clone, Is, Eq, PartialEq)]
pub enum AstVariable {
    String(String),
    Array(Vec<AstVariable>),
}

impl AstVariable {
    pub fn create_string(val: impl ToString) -> Self {
        Self::String(val.to_string())
    }
}

/// An AST node that represents a top-level statement.
#[derive(Debug, Clone, Is, Eq, PartialEq)]
pub enum AstStmtNode {
    DeclareVariable { name: String, value: AstVariable },
    SetExpr { name: String, value: AstExprNode },
    Comment(String),
    Test { expect: String, to_be: String },
}

impl AstStmtNode {
    pub fn create_declare_variable(name: impl ToString, value: AstVariable) -> Self {
        Self::DeclareVariable {
            name: name.to_string(),
            value,
        }
    }

    pub fn create_set_expr(name: impl ToString, value: AstExprNode) -> Self {
        Self::SetExpr {
            name: name.to_string(),
            value,
        }
    }

    pub fn create_test(expect: impl ToString, to_be: impl ToString) -> Self {
        Self::Test {
            expect: expect.to_string(),
            to_be: to_be.to_string(),
        }
    }
}
