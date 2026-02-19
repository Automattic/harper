use crate::{
    Token, TokenKind,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    patterns::{SingleTokenPattern, WhitespacePattern, prepositional_preceder},
};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct ToTooAdjectivePunct {
    expr: Box<dyn Expr>,
}

impl Default for ToTooAdjectivePunct {
    fn default() -> Self {
        let expr = SequenceExpr::optional(SequenceExpr::any_word().t_ws())
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
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        let to_index = tokens.iter().position(|t| {
            t.span
                .get_content(source)
                .eq_ignore_ascii_case_chars(&['t', 'o'])
        })?;

        let mut idx = to_index + 1;
        while idx < tokens.len() && tokens[idx].kind.is_whitespace() {
            idx += 1;
        }
        if idx >= tokens.len() {
            return None;
        }
        let adjective = &tokens[idx];
        if !adjective.kind.is_adjective() || !adjective.kind.is_positive_adjective() {
            return None;
        }
        if adjective.kind.is_preposition() {
            return None;
        }

        let prev_non_ws = tokens[..to_index].iter().rfind(|t| !t.kind.is_whitespace());
        if let Some(prev_token) = prev_non_ws
            && prepositional_preceder().matches_token(prev_token, source)
        {
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
        "Detects `to` before an adjective when followed by punctuation."
    }
}
