use crate::patterns::WhitespacePattern;
use crate::{
    Token, TokenKind,
    char_string::CharStringExt,
    expr::{AnchorEnd, Expr, SequenceExpr},
};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Sentence;

pub struct ToTooAdverb {
    expr: Box<dyn Expr>,
}

impl Default for ToTooAdverb {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then_kind_is_but_is_not_except(TokenKind::is_adverb, TokenKind::is_determiner, &["as"])
            .then_optional(WhitespacePattern)
            .then_any_of(vec![
                Box::new(SequenceExpr::default().then_kind_is_but_is_not_except(
                    TokenKind::is_punctuation,
                    |_| false,
                    &["`", "\"", "'", "“", "”", "‘", "’", "-", "–", "—"],
                )),
                Box::new(AnchorEnd),
            ]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ToTooAdverb {
    type Unit = Sentence;

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
        "Detects `to` before an adverb when it should be `too`."
    }
}
