use crate::{
    Lint, Token,
    expr::{AnchorEnd, Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct ForTheNthTime {
    expr: SequenceExpr,
}

impl Default for ForTheNthTime {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("for")
                .t_ws()
                .t_set(&[
                    "first",
                    "second",
                    "third",
                    "fourth",
                    "fifth",
                    "sixth",
                    "seventh",
                    "eighth",
                    "ninth",
                    "tenth",
                    "eleventh",
                    "twelfth",
                    "thirteenth",
                    "fourteenth",
                    "fifteenth",
                    "sixteenth",
                    "seventeenth",
                    "eighteenth",
                    "nineteenth",
                    "twentieth",
                    "thirtieth",
                    "fortieth",
                    "fiftieth",
                    "sixtieth",
                    "seventieth",
                    "eightieth",
                    "ninetieth",
                    "hundredth",
                    "thousandth",
                    "millionth",
                    "billionth",
                    // Special ordinals
                    "first",
                    // "last time" has many uses that don't require "the"
                    // "next time" has many uses that don't require "the"
                    "nth",
                    "umpteenth",
                    "zeroth",
                    "zeroeth",
                ])
                .t_ws()
                .t_aco("time")
                // .then(AnchorEnd),
                .then_any_of([
                    Box::new(AnchorEnd) as Box<dyn Expr>,
                    Box::new(SequenceExpr::whitespace().t_set(&["and", "but"])),
                ]),
        }
    }
}

impl ExprLinter for ForTheNthTime {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));

        Some(Lint {
            span: matched_tokens[0].span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::InsertAfter(vec![' ', 't', 'h', 'e'])],
            message: "The standard form of this construction requires the word 'the' to precede the ordinal.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects missing `the` for occasions like `on third time` -> `on the third time`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ForTheNthTime;

    #[test]
    fn fix_standing_up_opencloud() {
        assert_suggestion_result(
            "Lots of errors when standing up OpenCloud for first time.",
            ForTheNthTime::default(),
            "Lots of errors when standing up OpenCloud for the first time.",
        );
    }

    #[test]
    fn fix_for_second_time() {
        assert_suggestion_result(
            "SearchAnchor error on selecting an item for second time.",
            ForTheNthTime::default(),
            "SearchAnchor error on selecting an item for the second time.",
        );
    }

    #[test]
    fn fix_for_umpteenth_time() {
        assert_suggestion_result(
            "Having just accidentally \"closed and commented\", instead of \"commented\" for umpteenth time, I would like to suggest that",
            ForTheNthTime::default(),
            "Having just accidentally \"closed and commented\", instead of \"commented\" for the umpteenth time, I would like to suggest that",
        );
    }

    #[test]
    fn fix_for_nth_time_capitalized() {
        assert_suggestion_result(
            "sometimes breaks can be very long and you just want to pass your map, since you've failed it for Nth time",
            ForTheNthTime::default(),
            "sometimes breaks can be very long and you just want to pass your map, since you've failed it for the Nth time",
        );
    }

    #[test]
    fn fix_for_nth_time_lowercase() {
        assert_suggestion_result(
            "sometimes breaks can be very long and you just want to pass your map, since you've failed it for nth time",
            ForTheNthTime::default(),
            "sometimes breaks can be very long and you just want to pass your map, since you've failed it for the nth time",
        );
    }

    #[test]
    fn fix_for_millionth_time() {
        assert_suggestion_result(
            "try fixing bin name resolve for millionth time",
            ForTheNthTime::default(),
            "try fixing bin name resolve for the millionth time",
        );
    }

    #[test]
    fn fix_for_eleventh_time() {
        assert_suggestion_result(
            "DNF5 will fail after any package download fails for eleventh time",
            ForTheNthTime::default(),
            "DNF5 will fail after any package download fails for the eleventh time",
        );
    }

    #[test]
    fn flag_nth_time_followed_by_and() {
        assert_suggestion_result(
            "I am reading the schema documentation now for nth time and i do not confirm the main restriction",
            ForTheNthTime::default(),
            "I am reading the schema documentation now for the nth time and i do not confirm the main restriction",
        );
    }

    #[test]
    fn fix_seventh_and_eighth_time() {
        assert_suggestion_result(
            "Press the key for seventh time and the remainder is 7. The shell will print out the acceleration value;. Press the key for eighth time and the remainder is 0.",
            ForTheNthTime::default(),
            "Press the key for the seventh time and the remainder is 7. The shell will print out the acceleration value;. Press the key for the eighth time and the remainder is 0.",
        );
    }

    #[test]
    fn fix_eleventh_time() {
        assert_suggestion_result(
            "The number is cumulative, so e.g. for retries=10 , DNF5 will fail after any package download fails for eleventh time.",
            ForTheNthTime::default(),
            "The number is cumulative, so e.g. for retries=10 , DNF5 will fail after any package download fails for the eleventh time.",
        );
    }

    #[test]
    fn fix_thousandth_time() {
        assert_suggestion_result(
            "Why has no one told me before i tried for thousandth time?",
            ForTheNthTime::default(),
            "Why has no one told me before i tried for the thousandth time?",
        );
    }

    #[test]
    fn fix_sixth_time_but() {
        assert_suggestion_result(
            "Melissa McCarthy hosts for sixth time but laughs are sparse.",
            ForTheNthTime::default(),
            "Melissa McCarthy hosts for the sixth time but laughs are sparse.",
        );
    }

    // Handled false positives

    #[test]
    fn dont_flag_hyphenated_readers() {
        assert_no_lints(
            "and it felt like there is still a bit of a hurdle for first-time readers",
            ForTheNthTime::default(),
        );
    }

    #[test]
    fn dont_flag_unhyphenated_users() {
        assert_no_lints(
            "but just throwing this out as something less complicated for first time users",
            ForTheNthTime::default(),
        );
    }

    #[test]
    fn dont_flag_second_time_login() {
        assert_no_lints(
            "Google consent screen popup is not opening for second time login of another account",
            ForTheNthTime::default(),
        );
    }

    // As-yet unhandled cases - please feel free to implement these!
    // But take care to not introduce false positives!

    #[test]
    #[ignore = "not yet implemented"]
    fn should_fix_followed_by_even() {
        assert_suggestion_result(
            "Appium Android driver is un installing for second time even full reset is false",
            ForTheNthTime::default(),
            "Appium Android driver is un installing for the second time even full reset is false",
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn should_flag_nth_time_followed_by_in() {
        assert_suggestion_result(
            "but somewhat very tired to emphasize for Nth time in the minimap topic",
            ForTheNthTime::default(),
            "but somewhat very tired to emphasize for the Nth time in the minimap topic",
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn should_flag_fifth_time() {
        assert_suggestion_result(
            "I see with my naked eyes, no input field, but it says for fifth time there is input field, wtf, hpw to work with this shit?",
            ForTheNthTime::default(),
            "I see with my naked eyes, no input field, but it says for the fifth time there is input field, wtf, hpw to work with this shit?",
        );
    }
}
