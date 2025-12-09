mod ast;
mod error;
mod parsing;

pub use error::Error;
use parsing::parse_str;

use crate::expr::Expr;

pub fn nesl_to_expr(nesl_code: &str) -> Result<Box<dyn Expr>, Error> {
    let ast = parse_str(nesl_code)?;
    Ok(ast.to_expr())
}
