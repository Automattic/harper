use crate::char_string::CharStringExt;
use crate::{
    Token,
    expr::{Expr, SequenceExpr},
};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ToTooAdjVerbEdPunct {
    expr: Box<dyn Expr>,
}

impl Default for ToTooAdjVerbEdPunct {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then(|tok: &crate::Token, src: &[char]| {
                tok.kind.is_adjective()
                    && tok.kind.is_verb()
                    && !tok.kind.is_noun()
                    && tok
                        .span
                        .get_content(src)
                        .iter()
                        .collect::<String>()
                        .to_lowercase()
                        .ends_with("ed")
            })
            .then_sentence_terminator();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ToTooAdjVerbEdPunct {
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
        "Detects `to` before words that are adj/verb ending with `ed`, followed by punctuation."
    }
}
