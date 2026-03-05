use super::super::{ExprLinter, Lint, LintKind};
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::linting::Suggestion;
use crate::linting::expr_linter::Chunk;
use crate::{CharString, CharStringExt};
use crate::{Token, char_string::char_string};

pub struct ToHop {
    expr: Box<dyn Expr>,
}

impl Default for ToHop {
    fn default() -> Self {
        let pattern = SequenceExpr::word_set(&["hoping", "hoped", "hope"])
            .then_whitespace()
            .t_aco("on")
            .then_whitespace()
            .then_determiner()
            .then_whitespace()
            .then_word_set(&["airplane", "plane", "bus", "call", "train"]);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ToHop {
    fn to_correct(word: &str) -> Option<CharString> {
        Some(match word.to_lowercase().as_str() {
            "hoping" => char_string!("hopping"),
            "hoped" => char_string!("hopped"),
            "hope" => char_string!("hop"),
            _ => return None,
        })
    }
}

impl ExprLinter for ToHop {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_word = &matched_tokens[0];
        let word_chars = offending_word.span.get_content(source);
        let word = word_chars.to_string();
        let correct = Self::to_correct(&word)?;

        Some(Lint {
            span: offending_word.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                correct.to_vec(),
                word_chars,
            )],
            message: format!(
                "Did you mean to use {word} instead of {} in this context?",
                correct.to_string()
            ),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects incorrect usage of the words 'hoping,' 'hoped,' or 'hope' when referring to boarding or entering a mode of transportation. Suggests replacing them with the correct verb form such as 'hopping,' 'hopped,' or 'hop.'"
    }
}
