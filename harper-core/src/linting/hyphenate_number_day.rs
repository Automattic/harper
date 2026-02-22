use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, patterns::NominalPhrase};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct HyphenateNumberDay {
    expr: Box<dyn Expr>,
}

impl Default for HyphenateNumberDay {
    fn default() -> Self {
        let pattern = SequenceExpr::default()
            .then_number()
            .then_whitespace()
            .t_aco("day")
            .then_longest_of(vec![
                Box::new(SequenceExpr::whitespace().then(NominalPhrase)),
                Box::new(
                    SequenceExpr::default()
                        .then_hyphen()
                        .then_adjective()
                        .then_whitespace()
                        .then(NominalPhrase),
                ),
            ]);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for HyphenateNumberDay {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        let number = matched_tokens[0].kind.as_number()?;
        let space = &matched_tokens[1];

        Some(Lint {
            span: space.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith(vec!['-'])],
            message: format!("Use a hyphen in `{number}-day` when forming an adjectival compound."),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Ensures a hyphen is used in `X-day` when it is part of a compound adjective, such as `4-day work week`."
    }
}

#[cfg(test)]
mod tests {
    use super::HyphenateNumberDay;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn corrects_three_day_training() {
        assert_suggestion_result(
            "The company offers a 3 day training program.",
            HyphenateNumberDay::default(),
            "The company offers a 3-day training program.",
        );
    }

    #[test]
    fn corrects_five_day_challenge() {
        assert_suggestion_result(
            "Join the 5 day challenge to improve your skills.",
            HyphenateNumberDay::default(),
            "Join the 5-day challenge to improve your skills.",
        );
    }

    #[test]
    fn corrects_seven_day_plan() {
        assert_suggestion_result(
            "She followed a strict 7 day meal plan.",
            HyphenateNumberDay::default(),
            "She followed a strict 7-day meal plan.",
        );
    }

    #[test]
    fn does_not_correct_when_not_adjective() {
        assert_suggestion_result(
            "The seminar lasts for 2 days.",
            HyphenateNumberDay::default(),
            "The seminar lasts for 2 days.",
        );
    }

    #[test]
    fn corrects_varied_phrases() {
        assert_suggestion_result(
            "They implemented a new 6 day work schedule.",
            HyphenateNumberDay::default(),
            "They implemented a new 6-day work schedule.",
        );

        assert_suggestion_result(
            "Enroll in our 10 day fitness bootcamp!",
            HyphenateNumberDay::default(),
            "Enroll in our 10-day fitness bootcamp!",
        );
    }

    #[test]
    fn edge_case_day_long() {
        assert_suggestion_result(
            "The 4 day-long seminar was insightful.",
            HyphenateNumberDay::default(),
            "The 4-day-long seminar was insightful.",
        );
    }

    #[test]
    fn edge_case_plural_days() {
        assert_suggestion_result(
            "The trip was a fun 5 day experience.",
            HyphenateNumberDay::default(),
            "The trip was a fun 5-day experience.",
        );
    }

    #[test]
    fn ignores_spelled_out_numbers() {
        assert_suggestion_result(
            "We had a three day holiday.",
            HyphenateNumberDay::default(),
            "We had a three day holiday.",
        );
    }
}
