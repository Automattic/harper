use super::ast::{AstExprNode, AstStmtNode};

/// Optimizes the AST to use fewer nodes.
/// Returns whether an edit was made.
pub fn optimize(stmts: &mut Vec<AstStmtNode>) -> bool {
    let mut edit = false;

    for stmt in stmts {
        match stmt {
            AstStmtNode::ProduceExpr(node) => {
                if optimize_expr(node) {
                    edit = true;
                }
            }
            _ => (),
        }
    }

    edit
}

/// Optimizes the AST of the expression to use fewer nodes.
/// Returns whether an edit was made.
pub fn optimize_expr(ast: &mut AstExprNode) -> bool {
    let mut edit = false;
    match ast {
        AstExprNode::Not(child) => return optimize_expr(child),
        AstExprNode::Seq(children) => {
            if children.len() == 1 {
                *ast = children.pop().unwrap();
                edit = true;
            } else {
                children.iter_mut().for_each(|child| {
                    if optimize_expr(child) {
                        edit = true;
                    }
                });
            }
        }
        AstExprNode::Arr(children) => {
            if children.len() == 1 {
                *ast = children.pop().unwrap();
                edit = true;
            } else {
                children.iter_mut().for_each(|child| {
                    if optimize_expr(child) {
                        edit = true;
                    }
                });
            }
        }
        _ => (),
    }

    edit
}
