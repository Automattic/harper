use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct Hereby {
    expr: Box<dyn Expr>,
}

impl Default for Hereby {
    fn default() -> Self {
        let pattern = SequenceExpr::aco("here")
            .then_whitespace()
            .t_aco("by")
            .then_whitespace()
            .then_verb();

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for Hereby {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens[0..3].span()?;
        let orig_chars = span.get_content(source);
        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "hereby".chars().collect(),
                orig_chars,
            )],
            message: "Did you mean the closed compound `hereby`?".to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "`Here by` in some contexts should be `hereby`"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Hereby;

    #[test]
    fn declare() {
        assert_suggestion_result(
            "I here by declare this state to be free.",
            Hereby::default(),
            "I hereby declare this state to be free.",
        );
    }
}
