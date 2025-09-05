use crate::patterns::WhitespacePattern;
use crate::{
    Token, TokenKind,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ToTooAdjectivePunct {
    expr: Box<dyn Expr>,
}

impl Default for ToTooAdjectivePunct {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then_kind_is_but_is_not_except(
                TokenKind::is_adjective,
                TokenKind::is_verb,
                &["standard"],
            )
            .then_optional(WhitespacePattern)
            .then_sentence_terminator();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ToTooAdjectivePunct {
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
        "Detects `to` before an adjective when followed by punctuation."
    }
}
