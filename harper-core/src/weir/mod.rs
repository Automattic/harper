mod ast;
mod error;
mod optimize;
mod parsing;

use std::str::FromStr;

use crate::parsers::PlainEnglish;
pub use error::Error;
use is_macro::Is;
use parsing::{parse_expr_str, parse_str};
use strum_macros::{AsRefStr, EnumString};

use crate::expr::Expr;
use crate::linting::{Chunk, ExprLinter, Lint, LintKind, Linter, Suggestion};
use crate::{Document, Token, TokenStringExt};

use self::ast::{Ast, AstVariable};

pub fn weir_expr_to_expr(weir_code: &str) -> Result<Box<dyn Expr>, Error> {
    let ast = parse_expr_str(weir_code, true)?;
    Ok(ast.to_expr())
}

#[derive(Debug, Is, EnumString, AsRefStr)]
enum ReplacementStrategy {
    MatchCase,
    Exact,
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
    strategy: ReplacementStrategy,
    replacements: Vec<String>,
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
        let replacement_strat_name = "strategy";

        let expr = ast
            .get_expr(main_expr_name)
            .ok_or(Error::ExpectedVariableUndefined)?
            .to_expr();

        let description = ast
            .get_variable_value(description_name)
            .ok_or(Error::ExpectedVariableUndefined)?
            .as_string()
            .ok_or(Error::ExpectedDifferentVariableType)?
            .to_owned();

        let message = ast
            .get_variable_value(message_name)
            .ok_or(Error::ExpectedVariableUndefined)?
            .as_string()
            .ok_or(Error::ExpectedDifferentVariableType)?
            .to_owned();

        let replacement_val = ast
            .get_variable_value(replacement_name)
            .ok_or(Error::ExpectedVariableUndefined)?;

        let replacements = match replacement_val {
            AstVariable::String(s) => vec![s.to_owned()],
            AstVariable::Array(arr) => {
                let mut out = Vec::with_capacity(arr.len());
                for item in arr.iter().map(|v| {
                    v.as_string()
                        .cloned()
                        .ok_or(Error::ExpectedDifferentVariableType)
                }) {
                    let item = item?;
                    out.push(item);
                }
                out
            }
        };

        let replacement_strat_var = ast.get_variable_value(replacement_strat_name);
        let replacement_strat = if let Some(replacement_strat) = replacement_strat_var {
            let str = replacement_strat
                .as_string()
                .ok_or(Error::ExpectedDifferentVariableType)?;
            ReplacementStrategy::from_str(str)
                .ok()
                .ok_or(Error::InvalidReplacementStrategy)?
        } else {
            ReplacementStrategy::MatchCase
        };

        let lint_kind_var = ast.get_variable_value(lint_kind_name);
        let lint_kind = if let Some(lint_kind) = lint_kind_var {
            let str = lint_kind
                .as_string()
                .ok_or(Error::ExpectedDifferentVariableType)?;
            LintKind::from_string_key(str).ok_or(Error::InvalidLintKind)?
        } else {
            LintKind::Miscellaneous
        };

        let linter = WeirLinter {
            strategy: replacement_strat,
            ast,
            expr,
            lint_kind,
            description,
            message,
            replacements,
        };

        Ok(linter)
    }

    /// Counts the total number of tests defined.
    pub fn count_tests(&self) -> usize {
        self.ast.iter_tests().count()
    }

    /// Runs the tests defined in the source code, returning any failing results.
    pub fn run_tests(&mut self) -> Vec<TestResult> {
        fn matches_within_first_three_suggestions(
            lints: &[Lint],
            start: Vec<char>,
            to_be: &str,
        ) -> bool {
            fn dfs(lints: &[Lint], i: usize, current: Vec<char>, to_be: &str) -> bool {
                if i == lints.len() {
                    return current.into_iter().collect::<String>() == to_be;
                }

                let lint = &lints[i];

                if lint.suggestions.is_empty() {
                    return dfs(lints, i + 1, current, to_be);
                }

                for sug in lint.suggestions.iter().take(3) {
                    let mut next = current.clone();
                    sug.apply(lint.span, &mut next);
                    if dfs(lints, i + 1, next, to_be) {
                        return true;
                    }
                }

                false
            }

            dfs(lints, 0, start, to_be)
        }

        let ast = self.ast.clone();
        let mut results = Vec::new();

        for (expect, to_be) in ast.iter_tests() {
            let doc = Document::new_curated(expect, &PlainEnglish);
            let lints = self.lint(&doc);

            let mut default_output: Vec<char> = expect.chars().collect();
            for lint in &lints {
                if let Some(sug) = lint.suggestions.iter().take(3).next() {
                    sug.apply(lint.span, &mut default_output);
                }
            }
            let default_output: String = default_output.into_iter().collect();

            let passed =
                matches_within_first_three_suggestions(&lints, expect.chars().collect(), to_be);

            if !passed {
                results.push(TestResult {
                    expected: to_be.to_string(),
                    got: default_output,
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

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let orig = span.get_content(source);

        let suggestions = match self.strategy {
            ReplacementStrategy::MatchCase => self
                .replacements
                .iter()
                .map(|s| Suggestion::replace_with_match_case(s.chars().collect(), orig))
                .collect(),
            ReplacementStrategy::Exact => self
                .replacements
                .iter()
                .map(|r| Suggestion::ReplaceWith(r.chars().collect()))
                .collect(),
        };

        Some(Lint {
            span,
            lint_kind: self.lint_kind,
            suggestions,
            message: self.message.to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[cfg(test)]
pub mod tests {
    use super::{TestResult, WeirLinter};

    #[track_caller]
    pub fn assert_passes_all(linter: &mut WeirLinter) {
        assert_eq!(Vec::<TestResult>::new(), linter.run_tests());
    }

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
        assert_passes_all(&mut linter);
        assert_eq!(8, linter.count_tests());
    }

    #[test]
    fn g_suite() {
        let source = r#"
            set main [(G [Suite, Suit]), (Google Apps for Work)]
            declare message "Use the updated brand."
            declare description "`G Suite` or `Google Apps for Work` is now called `Google Workspace`"
            declare kind "Miscellaneous"
            declare becomes "Google Workspace"
            declare strategy "Exact"

            test "We migrated from G Suite last year." "We migrated from Google Workspace last year."
            test "This account is still labeled as Google Apps for Work." "This account is still labeled as Google Workspace."
            test "The pricing page mentions G Suit for legacy plans." "The pricing page mentions Google Workspace for legacy plans."
            test "New customers sign up for Google Workspace." "New customers sign up for Google Workspace."
            "#;

        let mut linter = WeirLinter::new(source).unwrap();

        assert_passes_all(&mut linter);
        assert_eq!(4, linter.count_tests());
    }
}
