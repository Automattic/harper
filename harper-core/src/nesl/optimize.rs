use super::ast::AstNode;

/// Optimizes the AST to use fewer nodes.
/// Returns whether an edit was made.
pub fn optimize(ast: &mut AstNode) -> bool {
    let mut edit = false;
    match ast {
        AstNode::Not(child) => return optimize(child),
        AstNode::Seq(children) => {
            if children.len() == 1 {
                *ast = children.pop().unwrap();
                edit = true;
            } else {
                children.iter_mut().for_each(|child| {
                    if optimize(child) {
                        edit = true;
                    }
                });
            }
        }
        AstNode::Arr(children) => {
            if children.len() == 1 {
                *ast = children.pop().unwrap();
                edit = true;
            } else {
                children.iter_mut().for_each(|child| {
                    if optimize(child) {
                        edit = true;
                    }
                });
            }
        }
        _ => (),
    }

    edit
}
