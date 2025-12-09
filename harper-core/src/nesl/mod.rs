mod ast;
mod error;
mod optimize;
mod parsing;

pub use error::Error;
use parsing::{parse_expr_str, parse_str};

use crate::expr::Expr;
use optimize::optimize;

pub fn nesl_expr_to_expr(nesl_code: &str) -> Result<Box<dyn Expr>, Error> {
    let ast = parse_expr_str(nesl_code, true)?;
    Ok(ast.to_expr())
}
