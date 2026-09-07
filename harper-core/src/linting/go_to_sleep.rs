use crate::{
    Lint, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

pub struct GoToSleep {
    expr: All,
}

impl Default for GoToSleep {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["go", "goes", "going", "gone", "went"])
                .t_ws()
                .then_word_seq(&["into", "sleep"])
                .but_not(
                    SequenceExpr::anything() // go
                        .t_any() // ws
                        .t_any() // into
                        .t_any() // ws
                        .t_any() // sleep
                        .then_any_of([
                            Box::new(
                                SequenceExpr::whitespace().then_any_of([
                                    Box::new(WordSet::new(&["mode", "state"])) as Box<dyn Expr>,
                                    Box::new(
                                        SequenceExpr::default()
                                            .then_adjective()
                                            .t_ws()
                                            .t_set(&["mode", "state"]),
                                    ),
                                ]),
                            ),
                            Box::new(SequenceExpr::default().then_slash().t_set(&[
                                "hibernate",
                                "hibernation",
                                "suspend",
                            ])),
                        ]),
                ),
        }
    }
}

impl ExprLinter for GoToSleep {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.get(2)?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "to",
                span.get_content(src),
            )],
            message: "Use `go to sleep` instead of `go into sleep`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `go into sleep` to `go to sleep`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::GoToSleep;

    #[test]
    fn fix_go_to_sleep() {
        assert_suggestion_result(
            "I changed sleep mode to s3 and now my computer cant go into sleep, it just shut down.",
            GoToSleep::default(),
            "I changed sleep mode to s3 and now my computer cant go to sleep, it just shut down.",
        );
    }

    #[test]
    fn fix_goes_into_sleep() {
        assert_suggestion_result(
            "On macbook with working Internet, close the lid so that mac goes into sleep.",
            GoToSleep::default(),
            "On macbook with working Internet, close the lid so that mac goes to sleep.",
        );
    }

    #[test]
    fn fix_going_into_sleep() {
        assert_suggestion_result(
            "Either pressing the power button on top of the Deck or going into Sleep from the Launcher freezes the Deck",
            GoToSleep::default(),
            "Either pressing the power button on top of the Deck or going to Sleep from the Launcher freezes the Deck",
        );
    }

    #[test]
    fn fix_gone_into_sleep() {
        assert_suggestion_result(
            "The last time this happened, it was after the computer had gone into sleep when I was away from my desk",
            GoToSleep::default(),
            "The last time this happened, it was after the computer had gone to sleep when I was away from my desk",
        );
    }

    #[test]
    fn fix_went_into_sleep() {
        assert_suggestion_result(
            "Broken UI rendering after macOS went into sleep",
            GoToSleep::default(),
            "Broken UI rendering after macOS went to sleep",
        );
    }

    #[test]
    fn dont_flag_go_into_sleep_state() {
        assert_no_lints(
            "When using Jobber, they either go into sleep state while still retaining memory",
            GoToSleep::default(),
        );
    }

    #[test]
    fn dont_flag_goes_into_sleep_mode() {
        assert_no_lints(
            "When the computer goes into sleep mode or the screen turns off, the program reports an error and closes.",
            GoToSleep::default(),
        );
    }

    #[test]
    fn dont_flag_went_into_sleep_suspend() {
        assert_no_lints(
            "The power LED remains ON even when it presumably went into sleep/suspend mode",
            GoToSleep::default(),
        );
    }

    #[test]
    fn dont_flag_goes_into_sleep_slash_hibernate() {
        assert_no_lints(
            "When the laptop goes into sleep/hibernate mode and they come back to it, it shows Netbird is connected",
            GoToSleep::default(),
        );
    }

    #[test]
    fn dont_flag_goes_into_sleep_slash_hibernation() {
        assert_no_lints(
            "Whenever system goes into sleep/hibernation and comes back, sometimes the driver will get \"stuck\"",
            GoToSleep::default(),
        );
    }
}
