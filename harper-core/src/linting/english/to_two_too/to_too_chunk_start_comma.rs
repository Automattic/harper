use crate::{
    Token,
    char_string::CharStringExt,
    expr::{AnchorStart, Expr, SequenceExpr},
    patterns::WhitespacePattern,
};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct ToTooChunkStartComma {
    expr: Box<dyn Expr>,
}

impl Default for ToTooChunkStartComma {
    fn default() -> Self {
        let expr = SequenceExpr::with(AnchorStart)
            .t_aco("to")
            .then_optional(WhitespacePattern)
            .then_comma();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ToTooChunkStartComma {
    type Unit = Chunk;

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
        "Detects `to` at the start of a clause before a comma."
    }
}
