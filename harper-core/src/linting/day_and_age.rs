use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

pub struct DayAndAge {
    expr: Box<dyn Expr>,
}

impl Default for DayAndAge {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::default()
                    .then_any_of(vec![
                        Box::new(SequenceExpr::default().then_preposition()),
                        Box::new(WordSet::new(&["it", "is"])),
                    ])
                    .t_ws()
                    .then_word_set(&["this", "these"])
                    .t_ws()
                    .then_word_set(&["day", "days"])
                    .t_ws()
                    .then_word_set(&["and", "in", "an", "on"])
                    .t_ws()
                    .then_word_set(&["age", "ages"])
                    .and_not(
                        SequenceExpr::word_set(&["in", "for"])
                            .t_any()
                            .t_aco("this")
                            .t_any()
                            .t_aco("day")
                            .t_any()
                            .t_aco("and")
                            .t_any()
                            .t_aco("age"),
                    ),
            ),
        }
    }
}

impl ExprLinter for DayAndAge {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Fixes wrong variants of the idiom `in this day and age`."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        all_toks: &[Token],
        src: &[char],
        _ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let toks = all_toks.iter().step_by(2).collect::<Vec<_>>();
        let spans = toks.iter().map(|t| t.span).collect::<Vec<_>>();
        let chars = spans.iter().map(|s| s.get_content(src)).collect::<Vec<_>>();

        let good: &[&[char]] = &[
            &['i', 'n'],
            &['t', 'h', 'i', 's'],
            &['d', 'a', 'y'],
            &['a', 'n', 'd'],
            &['a', 'g', 'e'],
        ];

        let bads: Vec<bool> = chars
            .iter()
            .zip(good.iter())
            .map(|(actual, &good)| !actual.eq_ignore_ascii_case_chars(good))
            .collect();

        if bads.iter().any(|&b| b) {
            return Some(Lint {
                span: all_toks.span()?,
                lint_kind: LintKind::Usage,
                suggestions: vec![Suggestion::replace_with_match_case_str(
                    "in this day and age",
                    all_toks.span()?.get_content(src),
                )],
                message: "The correct idiom is `in this day and age`".to_string(),
                ..Default::default()
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::DayAndAge;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    // True negatives

    #[test]
    fn allow_in_this_day_and_age() {
        assert_no_lints(
            "I do belive in this day and age with the amount of printer on the market ",
            DayAndAge::default(),
        );
    }

    #[test]
    fn for_this_day_and_age_seems_to_be_acceptable() {
        assert_no_lints(
            "As for my specs, I understand that my PC is quite underpowered for this day and age, but I'd say it's still within the hardware combos that ...",
            DayAndAge::default(),
        );
    }

    // True positives

    #[test]
    fn at_this_day_and_age() {
        assert_suggestion_result(
            "How would one add diagnostics to the compiler at this day and age?",
            DayAndAge::default(),
            "How would one add diagnostics to the compiler in this day and age?",
        );
    }

    #[test]
    fn by_this_day_in_age() {
        assert_suggestion_result(
            "Don't most people by this day in age, just have a spare laptop in their kitchens",
            DayAndAge::default(),
            "Don't most people in this day and age, just have a spare laptop in their kitchens",
        );
    }

    #[test]
    fn in_these_day_and_age() {
        assert_suggestion_result(
            "still don't come with load sharing components built in these day and age.",
            DayAndAge::default(),
            "still don't come with load sharing components built in this day and age.",
        );
    }

    #[test]
    fn in_these_days_and_age() {
        assert_suggestion_result(
            "But in these days and age floppies are replaced by USB flash drives.",
            DayAndAge::default(),
            "But in this day and age floppies are replaced by USB flash drives.",
        );
    }

    #[test]
    fn in_these_days_in_age() {
        assert_suggestion_result(
            "In these days in age, this is considered as 'heresy'.",
            DayAndAge::default(),
            "In this day and age, this is considered as 'heresy'.",
        );
    }

    #[test]
    fn in_this_day_an_age() {
        assert_suggestion_result(
            "but in this day an age things progressed a tad so might it be the time for increasing it?",
            DayAndAge::default(),
            "but in this day and age things progressed a tad so might it be the time for increasing it?",
        );
    }

    #[test]
    fn in_this_day_and_ages() {
        assert_suggestion_result(
            "or at least it should be in this day and ages",
            DayAndAge::default(),
            "or at least it should be in this day and age",
        );
    }

    #[test]
    fn in_this_day_in_age() {
        assert_suggestion_result(
            "or anything else that in this day in age is useful to have a reminder about",
            DayAndAge::default(),
            "or anything else that in this day and age is useful to have a reminder about",
        );
    }

    #[test]
    fn in_this_days_and_age() {
        assert_suggestion_result(
            "We as a whole realize that in this days and age being on social networking has got a sort of ...",
            DayAndAge::default(),
            "We as a whole realize that in this day and age being on social networking has got a sort of ...",
        );
    }

    #[test]
    fn is_this_day_and_age_typo() {
        assert_suggestion_result(
            "Agreed, dark mode is a necessity is this day and age.",
            DayAndAge::default(),
            "Agreed, dark mode is a necessity in this day and age.",
        );
    }

    #[test]
    fn it_this_day_and_age_typo() {
        assert_suggestion_result(
            "And it this day and age you really shouldn't but asking people to download random files",
            DayAndAge::default(),
            "And in this day and age you really shouldn't but asking people to download random files",
        );
    }

    #[test]
    fn of_this_day_and_age() {
        assert_suggestion_result(
            "it is completely incompatible with Juice Shop of this day and age",
            DayAndAge::default(),
            "it is completely incompatible with Juice Shop in this day and age",
        );
    }

    #[test]
    fn to_this_day_and_age() {
        assert_suggestion_result(
            "Still can't believe this has to be done in Safari to this day and age with responsive images",
            DayAndAge::default(),
            "Still can't believe this has to be done in Safari in this day and age with responsive images",
        );
    }

    // Unhandled edge cases

    #[test]
    #[ignore = "We don't yet handle missing preposition"]
    fn no_prep_this_day_in_age() {
        assert_suggestion_result(
            "if that is how gpu programming is still done this day in age then id have a very hard time seeing valhalla ever run on a gpu",
            DayAndAge::default(),
            "if that is how gpu programming is still done in this day and age then id have a very hard time seeing valhalla ever run on a gpu",
        );
    }

    #[test]
    #[ignore = "We don't yet handle missing preposition"]
    fn no_prep_these_days_and_ages() {
        assert_suggestion_result(
            "Btw I think you should write React the React Hooks way these days and ages, where you'll never see this keyword` again",
            DayAndAge::default(),
            "Btw I think you should write React the React Hooks way in this day and age, where you'll never see this keyword` again",
        );
    }

    #[test]
    #[ignore = "'Since' is a preposition but should be followed by 'in' rather than replaced"]
    fn is_since_a_preposition_or_not() {
        assert_suggestion_result(
            "and since these days and age storage is usually not a problem, I usually play it safe and just don't bother",
            DayAndAge::default(),
            "and since in this day and age storage is usually not a problem, I usually play it safe and just don't bother",
        );
    }
}
