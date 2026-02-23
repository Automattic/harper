use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::expr::WordExprGroup;
use itertools::Itertools;

use crate::{Lrc, Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct ThatWhich {
    expr: Box<dyn Expr>,
}

impl Default for ThatWhich {
    fn default() -> Self {
        let mut pattern = WordExprGroup::default();

        let matching_pattern = Lrc::new(
            SequenceExpr::any_capitalization_of("that")
                .then_whitespace()
                .then_any_capitalization_of("that"),
        );

        pattern.add("that", matching_pattern.clone());
        pattern.add("That", matching_pattern);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for ThatWhich {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let suggestion = format!(
            "{} which",
            matched_tokens[0]
                .span
                .get_content(source)
                .iter()
                .collect::<String>()
        )
        .chars()
        .collect_vec();

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Repetition,
            suggestions: vec![Suggestion::ReplaceWith(suggestion)],
            message: "“that that” sometimes means “that which”, which is clearer.".to_string(),
            priority: 126,
        })
    }

    fn description(&self) -> &'static str {
        "Repeating the word \"that\" is often redundant. The phrase `that which` is easier to read."
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::assert_lint_count;
    use super::ThatWhich;

    #[test]
    fn catches_lowercase() {
        assert_lint_count(
            "To reiterate, that that is cool is not uncool.",
            ThatWhich::default(),
            1,
        );
    }

    #[test]
    fn catches_different_cases() {
        assert_lint_count("That that is cool is not uncool.", ThatWhich::default(), 1);
    }

    #[test]
    fn likes_correction() {
        assert_lint_count(
            "To reiterate, that which is cool is not uncool.",
            ThatWhich::default(),
            0,
        );
    }
}
