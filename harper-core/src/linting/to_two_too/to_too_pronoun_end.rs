use crate::{
    Token, TokenKind,
    char_string::CharStringExt,
    expr::{AnchorEnd, AnchorStart, Expr, SequenceExpr},
    patterns::WhitespacePattern,
};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct ToTooPronounEnd {
    expr: Box<dyn Expr>,
}

impl Default for ToTooPronounEnd {
    fn default() -> Self {
        // Match at clause start or after punctuation to avoid cases like
        // "leave it to." where `it` is an object pronoun.
        let expr = SequenceExpr::any_of(vec![
            Box::new(SequenceExpr::with(AnchorStart)),
            Box::new(
                SequenceExpr::default()
                    .then_kind_is_but_is_not_except(
                        TokenKind::is_punctuation,
                        |_| false,
                        &["`", "\"", "'", "“", "”", "‘", "’"],
                    )
                    .then_optional(WhitespacePattern),
            ),
        ])
        .then_pronoun()
        .t_ws()
        .t_aco("to")
        .then_any_of(vec![
            Box::new(SequenceExpr::default().then_kind_is_but_is_not_except(
                TokenKind::is_punctuation,
                |_| false,
                &["`", "\"", "'", "“", "”", "‘", "’"],
            )),
            Box::new(AnchorEnd),
        ]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ToTooPronounEnd {
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
        "Detects `to` after a pronoun at clause end (e.g., `Me to!`)."
    }
}
