use crate::{
    Token,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ToTooDegreeWords {
    expr: Box<dyn Expr>,
}

impl Default for ToTooDegreeWords {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then_word_set(&["many", "much", "few"]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ToTooDegreeWords {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        let to_tok = tokens.iter().find(|t| {
            t.span
                .get_content(source)
                .eq_ignore_ascii_case_chars(&['t', 'o'])
        })?;

        Some(Lint {
            span: to_tok.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "too",
                to_tok.span.get_content(source),
            )],
            message: "Use `too` here to mean ‘also’ or an excessive degree.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Detects `to` used before degree words like `many`, `much`, or `few`."
    }
}
