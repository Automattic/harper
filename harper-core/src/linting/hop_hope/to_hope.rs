use super::super::{ExprLinter, Lint, LintKind};
use crate::char_string::char_string;
use crate::expr::Expr;
use crate::expr::MatchInfo;
use crate::expr::SequenceExpr;
use crate::linting::Suggestion;
use crate::patterns::WordSet;

pub struct ToHope {
    expr: Box<dyn Expr>,
}

impl Default for ToHope {
    fn default() -> Self {
        let pattern = SequenceExpr::default()
            .then_nominal()
            .then_whitespace()
            .then(WordSet::new(&["hop", "hopped"]))
            .then_whitespace()
            .then_nominal();

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for ToHope {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, match_info: MatchInfo<'_>, source: &[char]) -> Option<Lint> {
        let matched_tokens = match_info.matched_tokens;
        let offending_word = &matched_tokens[2];
        let word_chars = offending_word.span.get_content(source);

        Some(Lint {
            span: offending_word.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                char_string!("hope").to_vec(),
                word_chars,
            )],
            message: "Did you mean to use 'hope' instead of 'hop' in this context?".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects incorrect use of 'hop' when the correct verb 'hope' should be used in a sentence."
    }
}
