use crate::{
    Token, TokenKind,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    patterns::WhitespacePattern,
};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ToTooAdjectiveEnd {
    expr: Box<dyn Expr>,
}

impl Default for ToTooAdjectiveEnd {
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
            .then_optional(SequenceExpr::default().then_any_word())
            .then_optional(WhitespacePattern)
            .then_optional(SequenceExpr::default().then_punctuation());

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ToTooAdjectiveEnd {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        // Find the `to` token
        let to_index = tokens.iter().position(|t| {
            t.span
                .get_content(source)
                .eq_ignore_ascii_case_chars(&['t', 'o'])
        })?;

        // First non-whitespace after `to` should be the adjective
        let mut idx = to_index + 1;
        while idx < tokens.len() && tokens[idx].kind.is_whitespace() {
            idx += 1;
        }
        if idx >= tokens.len() || !tokens[idx].kind.is_adjective() {
            return None;
        }

        // Find the next non-whitespace after the adjective
        let mut j = idx + 1;
        while j < tokens.len() && tokens[j].kind.is_whitespace() {
            j += 1;
        }

        let should_lint = if j >= tokens.len() {
            true
        } else if tokens[j].kind.is_punctuation() {
            let punct: String = tokens[j].span.get_content(source).iter().collect();
            !matches!(
                punct.as_str(),
                "`" | "\"" | "'" | "“" | "”" | "‘" | "’" | "-" | "–" | "—"
            )
        } else {
            false
        };

        if !should_lint {
            return None;
        }

        let to_tok = &tokens[to_index];

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
        "Detects `to` before an adjective when no word follows (end or punct)."
    }
}
