use crate::CharStringExt;
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::patterns::WordSet;
use crate::{
    Token, TokenStringExt,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct OnFloor {
    expr: Box<dyn Expr>,
}

impl Default for OnFloor {
    fn default() -> Self {
        let preposition = WordSet::new(&["in", "at"]);

        let pattern = SequenceExpr::default()
            .then(preposition)
            .t_ws()
            .t_aco("the")
            .t_ws()
            .t_any()
            .t_ws()
            .t_aco("floor");

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for OnFloor {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let incorrect_preposition = matched_tokens[0..1].span()?.get_content(source).to_string();
        let span = matched_tokens[0..1].span()?;

        Some(Lint {
            lint_kind: LintKind::WordChoice,
            span,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "on",
                span.get_content(source),
            )],
            message: format!(
                "Corrects `{incorrect_preposition}` to `on` when talking about position inside a building",
            )
            .to_string(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "This rule identifies incorrect uses of the prepositions `in` or `at` when referring to locations inside a building and recommends using `on the floor` instead."
    }
}

#[cfg(test)]
mod tests {
    use super::OnFloor;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn not_lint_with_correct_phrase() {
        assert_lint_count(
            "I'm living on the 3rd floor of a building.",
            OnFloor::default(),
            0,
        );
    }

    #[test]
    fn lint_with_in() {
        assert_suggestion_result(
            "I'm living in the 3rd floor of a building.",
            OnFloor::default(),
            "I'm living on the 3rd floor of a building.",
        );
    }

    #[test]
    fn lint_with_at() {
        assert_suggestion_result(
            "I'm living at the second floor of a building.",
            OnFloor::default(),
            "I'm living on the second floor of a building.",
        );
    }

    #[test]
    fn in_the_start_of_sentence() {
        assert_suggestion_result(
            "In the 3rd floor of a building.",
            OnFloor::default(),
            "On the 3rd floor of a building.",
        );
    }

    #[test]
    fn at_the_start_of_sentence() {
        assert_suggestion_result(
            "At the second floor of a building.",
            OnFloor::default(),
            "On the second floor of a building.",
        );
    }
}
