use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::{InflectionOfBe, Word, WordSet},
};

pub struct CraveFor {
    expr: FirstMatchOf,
}

impl Default for CraveFor {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new(vec![
                Box::new(
                    SequenceExpr::word_set(&["crave", "craved", "craves"])
                        .t_ws()
                        .t_aco("for"),
                ),
                Box::new(
                    SequenceExpr::any_of(vec![
                        Box::new(InflectionOfBe::default()),
                        Box::new(WordSet::new(&[
                            "i'm", "we're", "you're", "he's", "she's", "it's", "they're",
                        ])),
                    ])
                    .t_ws()
                    .t_aco("craving")
                    .t_ws()
                    .t_aco("for")
                    .but_not(Word::new("being")),
                ),
            ]),
        }
    }
}

impl ExprLinter for CraveFor {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let span = toks[toks.len() - 2..].span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::Remove],
            message: "The verb `crave` should not be followed by `for`.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "There should be no `for` after the verb `crave`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::CraveFor;

    // Crave for

    #[test]
    fn fix_crave_for() {
        assert_suggestion_result(
            "We crave for this inference code.",
            CraveFor::default(),
            "We crave this inference code.",
        );
    }

    #[test]
    fn fix_ever_craved_for() {
        assert_suggestion_result(
            "Ever craved for a tool to monitor what files are growing faster than other ones?",
            CraveFor::default(),
            "Ever craved a tool to monitor what files are growing faster than other ones?",
        );
    }

    #[test]
    fn fix_craves_for() {
        assert_suggestion_result(
            "Defending coders is the main talent everyone craves for.",
            CraveFor::default(),
            "Defending coders is the main talent everyone craves.",
        );
    }

    // Be craving for

    #[test]
    fn fix_am_craving_for() {
        assert_suggestion_result(
            "but hopefully it's the avenue that leads me to what am craving for !",
            CraveFor::default(),
            "but hopefully it's the avenue that leads me to what am craving !",
        );
    }

    #[test]
    fn fix_are_craving_for() {
        assert_suggestion_result(
            "Many of my users are craving for a timepicker in dropdown format :)",
            CraveFor::default(),
            "Many of my users are craving a timepicker in dropdown format :)",
        );
    }

    #[test]
    fn fix_be_craving_for() {
        assert_suggestion_result(
            "It offers you the no-frills experience for Hugo that you might be craving for.",
            CraveFor::default(),
            "It offers you the no-frills experience for Hugo that you might be craving.",
        );
    }

    #[test]
    fn fix_been_craving_for() {
        assert_suggestion_result(
            "And this is something the community has been craving for and splitting up the ecosystem",
            CraveFor::default(),
            "And this is something the community has been craving and splitting up the ecosystem",
        );
    }

    #[test]
    fn fix_i_m_craving_for() {
        assert_suggestion_result(
            "I'm craving for Hazel engine devlog man...",
            CraveFor::default(),
            "I'm craving Hazel engine devlog man...",
        );
    }

    #[test]
    fn fix_is_craving_for() {
        assert_suggestion_result(
            "Now everyone is craving for this level of abstraction running on GPUs.",
            CraveFor::default(),
            "Now everyone is craving this level of abstraction running on GPUs.",
        );
    }

    #[test]
    fn fix_was_craving_for() {
        assert_suggestion_result(
            "Yeah, I was craving for some splinter cell action, did a playthrough of CT",
            CraveFor::default(),
            "Yeah, I was craving some splinter cell action, did a playthrough of CT",
        );
    }

    #[test]
    fn fix_were_craving_for() {
        assert_suggestion_result(
            "is this the \"freedom\" and the \"flexibility\" we were craving for compared to the Arduino folks way of doing things uC?",
            CraveFor::default(),
            "is this the \"freedom\" and the \"flexibility\" we were craving compared to the Arduino folks way of doing things uC?",
        );
    }

    // Avoid false positives

    #[test]
    fn dont_flag_one_of_them_being_craving() {
        assert_no_lints(
            "The Second Noble Truth identifies three aspects of craving, one of them being craving for non-being.",
            CraveFor::default(),
        );
    }
}
