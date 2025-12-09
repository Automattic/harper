mod ast;
mod error;
mod optimize;
mod parsing;

pub use error::Error;
use parsing::{parse_expr_str, parse_str};

use crate::expr::Expr;
use crate::linting::{Chunk, ExprLinter, Lint, LintKind, Linter};
use crate::{Token, TokenStringExt};

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

    Ok(Box::new(NeslLinter {
        expr,
        lint_kind,
        description: description.to_owned(),
        message: message.to_owned(),
    }))
}

#[cfg(test)]
mod tests {
    use super::nesl_to_linter;

    #[test]
    fn simple_right_click_linter() {
        let source = r#"
            let main [right, middle, left] [click, clicked]
            declare message Hyphenate this mouse command
            declare description Hyphenates right-click style mouse commands.
            declare kind Punctuation

            declare hyphenates_basic_command Right click the icon.
            declare hyphenates_with_preposition Please right click on the link.
            declare hyphenates_past_tense They right clicked the submit button.
            declare hyphenates_gerund Right clicking the item highlights it.
            declare hyphenates_plural_noun Right clicks are tracked in the log.
            declare hyphenates_all_caps He RIGHT CLICKED the file.
            declare hyphenates_left_click Left click the checkbox.
            declare hyphenates_middle_click Middle click to open in a new tab.
            declare allows_hyphenated_form Right-click the icon.
            declare ignores_unrelated_right_and_click Click the right button to continue.
            
            # Maybe syntax like this?

            expect "test case" to be "result"

            "#;

        let linter = nesl_to_linter(source).unwrap();
    }
}
