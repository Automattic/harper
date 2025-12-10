mod ast;
mod error;
mod optimize;
mod parsing;

use crate::Document;
use crate::parsers::PlainEnglish;
pub use error::Error;
use parsing::{parse_expr_str, parse_str};

use crate::expr::Expr;
use crate::linting::{Chunk, ExprLinter, Lint, LintKind, Linter};
use crate::{Document, Token, TokenStringExt};

pub fn nesl_expr_to_expr(nesl_code: &str) -> Result<Box<dyn Expr>, Error> {
    let ast = parse_expr_str(nesl_code, true)?;
    Ok(ast.to_expr())
}

struct NeslLinter {
    expr: Box<dyn Expr>,
    description: String,
    message: String,
    lint_kind: LintKind,
}

impl ExprLinter for NeslLinter {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: self.lint_kind,
            suggestions: vec![],
            message: self.message.to_owned(),
            priority: 127,
        })
    }

    fn description(&self) -> &str {
        &self.description
    }
}

pub fn nesl_to_linter(nesl_code: &str) -> Result<Box<dyn Linter>, Error> {
    let ast = parse_str(nesl_code, true)?;

    let main_expr_name = "main";
    let description_name = "description";
    let message_name = "message";
    let lint_kind_name = "kind";

    let exclusions = [
        main_expr_name,
        description_name,
        message_name,
        lint_kind_name,
    ];

    let mut examples = Vec::new();
    for (name, value) in ast.iter_variable_values() {
        if !exclusions.contains(&name) {
            examples.push(value.to_string());
        }
    }

    let expr = ast
        .get_expr(main_expr_name)
        .ok_or(Error::ExpectedVariableUndefined)?
        .to_expr();

    let description = ast
        .get_variable_value(description_name)
        .ok_or(Error::ExpectedVariableUndefined)?;

    let message = ast
        .get_variable_value(message_name)
        .ok_or(Error::ExpectedVariableUndefined)?;

    let lint_kind = ast
        .get_variable_value(lint_kind_name)
        .ok_or(Error::ExpectedVariableUndefined)?;
    let lint_kind = LintKind::from_string_key(lint_kind).ok_or(Error::InvalidLintKind)?;

    let mut linter = NeslLinter {
        expr,
        lint_kind,
        description: description.to_owned(),
        message: message.to_owned(),
    };

    Ok(Box::new(linter))
}

#[cfg(test)]
mod tests {
    use super::nesl_to_linter;

    #[test]
    fn simple_right_click_linter() {
        let source = r#"
            set main [right, middle, left] [click, clicked]
            declare message Hyphenate this mouse command
            declare description Hyphenates right-click style mouse commands.
            declare kind Punctuation

            test "Right click the icon." "Right-click the icon."
            test "Please right click on the link." "Please right-click on the link."
            test "They right clicked the submit button." "They right-clicked the submit button."
            test "Right clicking the item highlights it." "Right-clicking the item highlights it."
            test "Right clicks are tracked in the log." "Right-clicks are tracked in the log."
            test "He RIGHT CLICKED the file." "He RIGHT-CLICKED the file."
            test "Left click the checkbox." "Left-click the checkbox."
            test "Middle click to open in a new tab." "Middle-click to open in a new tab."

            "#;

        let linter = nesl_to_linter(source).unwrap();
    }
}
