use crate::{
    Lint, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct TheLastDays {
    expr: All,
}

impl Default for TheLastDays {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_preposition()
                .t_ws()
                .t_aco("the")
                .t_ws()
                .t_aco("last")
                .t_ws()
                .then_word_set(&[
                    "seconds", "minutes", "hours", "days", "weeks", "months", "years", "decades",
                ])
                .but_not(
                    SequenceExpr::anything() // prep 0
                        .t_any() // _
                        .t_any() // the 2
                        .t_any() // _
                        .t_any() // last 4
                        .t_any() // _
                        .t_any() // days 6
                        .t_ws() // _
                        .t_aco("of"),
                ),
        }
    }
}

impl ExprLinter for TheLastDays {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let last_idx = 4;

        let suggestions = vec![Suggestion::InsertAfter(" few".chars().collect())];
        let message =
            "To speak about recent time use `the last few` rather than just `the last`.".to_owned();

        Some(Lint {
            span: toks[last_idx].span,
            lint_kind: LintKind::Usage,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `in the last days` to `in the last few days` and related errors."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::TheLastDays;

    #[test]
    fn fix_in_the_last_minutes() {
        assert_suggestion_result(
            "Required Files changed in the last minutes.",
            TheLastDays::default(),
            "Required Files changed in the last few minutes.",
        )
    }

    #[test]
    fn fix_for_the_last_days() {
        assert_suggestion_result(
            "The nightly docs CI job and the job on the individual commits has been failing for the last days",
            TheLastDays::default(),
            "The nightly docs CI job and the job on the individual commits has been failing for the last few days",
        );
    }

    #[test]
    fn fix_during_the_last_weeks() {
        assert_suggestion_result(
            "no, i was not in germany during the last weeks!",
            TheLastDays::default(),
            "no, i was not in germany during the last few weeks!",
        );
    }

    #[test]
    fn fix_during_the_last_months() {
        assert_suggestion_result(
            "GroupDocs.Viewer.dll increasing rapidly in size during the last months",
            TheLastDays::default(),
            "GroupDocs.Viewer.dll increasing rapidly in size during the last few months",
        );
    }

    #[test]
    fn fix_in_the_last_years() {
        assert_suggestion_result(
            "In the last years I've been asked multiple times about the comparison between raylib and SDL libraries.",
            TheLastDays::default(),
            "In the last few years I've been asked multiple times about the comparison between raylib and SDL libraries.",
        );
    }

    #[test]
    fn fix_during_the_last_decades() {
        assert_suggestion_result(
            "During the last decades they have been used successfully as digital music archive management tools",
            TheLastDays::default(),
            "During the last few decades they have been used successfully as digital music archive management tools",
        );
    }

    #[test]
    fn dont_flag_seconds_without_preposition() {
        assert_no_lints(
            "It grabs the last seconds from the song and continues the song only from that data.",
            TheLastDays::default(),
        )
    }

    #[test]
    fn dont_flag_minutes_without_preposition() {
        assert_no_lints(
            "so the tracker can count the last minutes it was seeded",
            TheLastDays::default(),
        )
    }

    #[test]
    fn dont_flag_days_without_preposition() {
        assert_no_lints(
            "Wrong date being returned during the last days of year",
            TheLastDays::default(),
        )
    }

    #[test]
    #[ignore = "not after a preposition but is a mistake...or maybe ambiguous"]
    fn fix_spent_the_last_hours() {
        assert_suggestion_result(
            "And from what I spent the last hours investigating there's no way to change this behavior",
            TheLastDays::default(),
            "And from what I spent the last few hours investigating there's no way to change this behavior",
        );
    }

    #[test]
    #[ignore = "not after a preposition but is a mistake"]
    fn fix_4_times_the_last_months() {
        assert_suggestion_result(
            "We have been affected 4 different times the last months due to upgrades coming from \"async\" mode",
            TheLastDays::default(),
            "We have been affected 4 different times the last months due to upgrades coming from \"async\" mode",
        );
    }

    #[test]
    fn dont_flag_last_seconds_of() {
        assert_no_lints(
            "But if you have specific trouble at the last seconds of the stream, try this PR",
            TheLastDays::default(),
        );
    }
}
