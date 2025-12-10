mod ast;
mod error;
mod optimize;
mod parsing;

use crate::parsers::PlainEnglish;
pub use error::Error;
use parsing::{parse_expr_str, parse_str};

use crate::expr::Expr;
use crate::linting::{Chunk, ExprLinter, Lint, LintKind, Linter, Suggestion};
use crate::{Document, Token, TokenStringExt};

use self::ast::Ast;

pub fn weir_expr_to_expr(weir_code: &str) -> Result<Box<dyn Expr>, Error> {
    let ast = parse_expr_str(weir_code, true)?;
    Ok(ast.to_expr())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestResult {
    expected: String,
    got: String,
}

pub struct WeirLinter {
    expr: Box<dyn Expr>,
    description: String,
    message: String,
    replacement: String,
    lint_kind: LintKind,
    ast: Ast,
}

impl WeirLinter {
    pub fn new(weir_code: &str) -> Result<WeirLinter, Error> {
        let ast = parse_str(weir_code, true)?;

        let main_expr_name = "main";
        let description_name = "description";
        let message_name = "message";
        let lint_kind_name = "kind";
        let replacement_name = "becomes";

        let expr = ast
            .get_expr(main_expr_name)
            .ok_or(Error::ExpectedVariableUndefined)?
            .to_expr();

        let description = ast
            .get_variable_value(description_name)
            .ok_or(Error::ExpectedVariableUndefined)?
            .to_string();

        let message = ast
            .get_variable_value(message_name)
            .ok_or(Error::ExpectedVariableUndefined)?
            .to_string();

        let replacement = ast
            .get_variable_value(replacement_name)
            .ok_or(Error::ExpectedVariableUndefined)?
            .to_string();

        let lint_kind = ast
            .get_variable_value(lint_kind_name)
            .ok_or(Error::ExpectedVariableUndefined)?;
        let lint_kind = LintKind::from_string_key(lint_kind).ok_or(Error::InvalidLintKind)?;

        let linter = WeirLinter {
            ast,
            expr,
            lint_kind,
            description,
            message,
            replacement,
        };

        Ok(linter)
    }

    /// Counts the total number of tests defined.
    pub fn count_tests(&self) -> usize {
        self.ast.iter_tests().count()
    }

    /// Runs the tests defined in the source code, returning any failing results.
    pub fn run_tests(&mut self) -> Vec<TestResult> {
        let ast = self.ast.clone();
        let mut results = Vec::new();

        for (expect, to_be) in ast.iter_tests() {
            let doc = Document::new_curated(expect, &PlainEnglish);
            let lints = self.lint(&doc);

            let mut output = expect.chars().collect();

            for lint in lints {
                if let Some(sug) = lint.suggestions.first() {
                    sug.apply(lint.span, &mut output);
                }
            }

            let output: String = output.into_iter().collect();

            if !output.chars().eq(to_be.chars()) {
                results.push(TestResult {
                    expected: to_be.to_string(),
                    got: output,
                });
            }
        }

        results
    }
}

impl ExprLinter for WeirLinter {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: self.lint_kind,
            suggestions: vec![Suggestion::ReplaceWith(self.replacement.chars().collect())],
            message: self.message.to_owned(),
            priority: 127,
        })
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[cfg(test)]
mod tests {
    use super::{TestResult, WeirLinter};

    #[test]
    fn simple_right_click_linter() {
        let source = r#"
            set main <([right, middle, left] $click), ( )>
            declare message "Hyphenate this mouse command"
            declare description "Hyphenates right-click style mouse commands."
            declare kind "Punctuation"
            declare becomes "-"

            test "Right click the icon." "Right-click the icon."
            test "Please right click on the link." "Please right-click on the link."
            test "They right clicked the submit button." "They right-clicked the submit button."
            test "Right clicking the item highlights it." "Right-clicking the item highlights it."
            test "Right clicks are tracked in the log." "Right-clicks are tracked in the log."
            test "He RIGHT CLICKED the file." "He RIGHT-CLICKED the file."
            test "Left click the checkbox." "Left-click the checkbox."
            test "Middle click to open in a new tab." "Middle-click to open in a new tab."
            "#;

        let mut linter = WeirLinter::new(source).unwrap();

        assert_eq!(Vec::<TestResult>::new(), linter.run_tests())
    }
}
