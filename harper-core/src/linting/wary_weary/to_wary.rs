use super::super::{ExprLinter, Lint, LintKind};
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::linting::Suggestion;
use crate::linting::expr_linter::Chunk;
use crate::{Token, char_string::char_string};

/// Detects `weary` used where `wary` (cautious) is intended.
pub struct ToWary {
    expr: Box<dyn Expr>,
}

impl Default for ToWary {
    fn default() -> Self {
        // "weary eye" (singular) is nearly always a mistake for the fixed idiom
        // "wary eye" (a cautious, watchful eye). The plural "weary eyes" is
        // deliberately excluded: it is a common, correct collocation for "tired
        // eyes", so flagging it would produce frequent false positives.
        let pattern = SequenceExpr::word_set(&["weary"])
            .then_whitespace()
            .then_word_set(&["eye"]);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for ToWary {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_word = &matched_tokens[0];
        let word_chars = offending_word.get_ch(source);

        Some(Lint {
            span: offending_word.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                char_string!("wary").to_vec(),
                word_chars,
            )],
            message: "Did you mean `wary` (cautious) rather than `weary` (tired) here?".to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects `weary` (tired) used where `wary` (cautious) is intended."
    }
}
