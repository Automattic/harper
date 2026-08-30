use super::super::{ExprLinter, Lint, LintKind};
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::linting::Suggestion;
use crate::linting::expr_linter::Chunk;
use crate::{Token, char_string::char_string};

/// Detects `wary` used where `weary` (tired) is intended.
pub struct ToWeary {
    expr: Box<dyn Expr>,
}

impl Default for ToWeary {
    fn default() -> Self {
        // "bone weary" and "world weary" are fixed idioms meaning exhausted;
        // "wary" in these phrases is a mistake for "weary".
        let pattern = SequenceExpr::word_set(&["bone", "world"])
            .then_whitespace()
            .then_word_set(&["wary"]);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for ToWeary {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_word = &matched_tokens[2];
        let word_chars = offending_word.get_ch(source);

        Some(Lint {
            span: offending_word.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                char_string!("weary").to_vec(),
                word_chars,
            )],
            message: "Did you mean `weary` (tired) rather than `wary` (cautious) here?".to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects `wary` used where `weary` (tired) is intended, as in `bone weary` or `world weary`."
    }
}
