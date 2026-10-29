use crate::{
    Lint, Token, TokenStringExt,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct AnArmAndALeg {
    expr: All,
}

impl Default for AnArmAndALeg {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set([
                "charge", "charged", "charges", "charging", "cost", "costed", "costing", "costs",
                "paid", "pay", "payed", "paying", "pays",
            ])
            .t_ws()
            .then_any_of([
                Box::new(
                    SequenceExpr::optional(SequenceExpr::word_set(["a", "an", "the"]).t_ws())
                        .t_set(["arm", "arms"])
                        .t_ws()
                        .t_aco("and")
                        .t_ws()
                        .then_optional(SequenceExpr::word_set(["a", "an", "the"]).t_ws())
                        .t_set(["leg", "legs"]),
                ),
                Box::new(
                    SequenceExpr::optional(SequenceExpr::word_set(["a", "an", "the"]).t_ws())
                        .t_set(["leg", "legs"])
                        .t_ws()
                        .t_aco("and")
                        .t_ws()
                        .then_optional(SequenceExpr::word_set(["a", "an", "the"]).t_ws())
                        .t_set(["arm", "arms"]),
                ),
            ])
            .but_not(
                SequenceExpr::anything()
                    .t_any()
                    .then_word_seq(&["an", "arm", "and", "a", "leg"]),
            ),
        }
    }
}

impl ExprLinter for AnArmAndALeg {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let toks = &toks[2..];
        let span = toks.span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "an arm and a leg",
                span.get_content(src),
            )],
            message: "Use the standard idiom `an arm and a leg` instead.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects wrong variants of the idiom `an arm and a leg`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AnArmAndALeg;

    #[test]
    fn paying_a_leg_and_an_arm() {
        assert_suggestion_result(
            "I am sick and tired of paying a leg and an arm in NYC taxes.",
            AnArmAndALeg::default(),
            "I am sick and tired of paying an arm and a leg in NYC taxes.",
        );
    }

    #[test]
    fn charge_leg_and_arm() {
        assert_suggestion_result(
            "they just “fix things” but charge leg and arm for their services",
            AnArmAndALeg::default(),
            "they just “fix things” but charge an arm and a leg for their services",
        );
    }

    #[test]
    fn charging_arm_and_leg() {
        assert_suggestion_result(
            "higher education institute charging arm and leg for people to network",
            AnArmAndALeg::default(),
            "higher education institute charging an arm and a leg for people to network",
        );
    }

    #[test]
    fn cost_an_arm_and_leg() {
        assert_suggestion_result(
            "Professional workstations have always cost an arm and leg - both arms and both legs usually",
            AnArmAndALeg::default(),
            "Professional workstations have always cost an arm and a leg - both arms and both legs usually",
        );
    }

    #[test]
    fn pay_arm_and_a_leg() {
        assert_suggestion_result(
            "than continuing to pay arm and a leg for eternity",
            AnArmAndALeg::default(),
            "than continuing to pay an arm and a leg for eternity",
        );
    }

    #[test]
    fn costs_arm_and_leg() {
        assert_suggestion_result(
            "Using your own analogy car repair costs arm and leg with the dealer vs diy",
            AnArmAndALeg::default(),
            "Using your own analogy car repair costs an arm and a leg with the dealer vs diy",
        );
    }

    #[test]
    fn charge_an_arm_and_leg() {
        assert_suggestion_result(
            "They manage to charge an arm and leg for regular things that a lot of the world makes",
            AnArmAndALeg::default(),
            "They manage to charge an arm and a leg for regular things that a lot of the world makes",
        );
    }

    #[test]
    fn dont_flag_removal() {
        assert_no_lints(
            "As in, they require removal of an arm and a leg.",
            AnArmAndALeg::default(),
        );
    }
}
