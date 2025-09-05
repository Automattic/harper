use crate::{
    Token, TokenKind,
    char_string::CharStringExt,
    expr::{AnchorEnd, Expr, SequenceExpr},
};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ToTooDegreeWords {
    expr: Box<dyn Expr>,
}

impl Default for ToTooDegreeWords {
    fn default() -> Self {
        // Only flag `to` before degree words when the phrase ends the clause
        // (punctuation or end). Avoids false positives like "connected to many X".
        let expr = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then_word_set(&["many", "much", "few"])
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
