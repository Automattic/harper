use crate::{
    Token,
    expr::{DurationExpr, Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk},
    token_string_ext::TokenStringExt,
};

pub struct InTimeFromNow {
    expr: SequenceExpr,
}

impl Default for InTimeFromNow {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("in")
                .t_ws()
                .then_optional(
                    SequenceExpr::word_set(&[
                        "about",
                        "almost",
                        "approximately",
                        "around",
                        "circa",
                        "exactly",
                        "just",
                        "maybe",
                        "nearly",
                        "only",
                        "perhaps",
                        "precisely",
                        "probably",
                        "roughly",
                    ])
                    .t_ws(),
                )
                .then(DurationExpr)
                .t_ws()
                .t_aco("from")
                .t_ws()
                .t_aco("now"),
        }
    }
}

impl ExprLinter for InTimeFromNow {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let without_in: Vec<char> = toks[2..].span()?.get_content(src).to_vec();
        let without_from_now: Vec<char> = toks[..toks.len() - 4].span()?.get_content(src).to_vec();

        let template_chars = toks.span()?.get_content(src);

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Redundancy,
            message: "Avoid redundancy by using either `in [period of time]` or `[period of time] from now`, but not both together.".to_string(),
            suggestions: vec![
                Suggestion::replace_with_match_case(without_in, template_chars),
                Suggestion::replace_with_match_case(without_from_now, template_chars),
            ],
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Checks for redundant use of `in` before [period of time] together with `from now` after it."
    }
}

#[cfg(test)]
mod tests {
    use super::InTimeFromNow;
    use crate::linting::tests::assert_good_and_bad_suggestions;

    #[test]
    fn in_three_years_from_now() {
        assert_good_and_bad_suggestions(
            "Closing this issue now to prevent it from still being open in three years from now.",
            InTimeFromNow::default(),
            &[
                "Closing this issue now to prevent it from still being open in three years.",
                "Closing this issue now to prevent it from still being open three years from now.",
            ],
            &[],
        );
    }

    #[test]
    fn in_2_seconds_from_now() {
        assert_good_and_bad_suggestions(
            "The task will be executed in 2 seconds from now.",
            InTimeFromNow::default(),
            &[
                "The task will be executed 2 seconds from now.",
                "The task will be executed in 2 seconds.",
            ],
            &[],
        );
    }

    #[test]
    fn in_three_weeks_from_now() {
        assert_good_and_bad_suggestions(
            "I have created a pull request, which can be merged in three weeks from now",
            InTimeFromNow::default(),
            &[
                "I have created a pull request, which can be merged in three weeks",
                "I have created a pull request, which can be merged three weeks from now",
            ],
            &[],
        );
    }

    #[test]
    fn in_2_hours_from_now() {
        assert_good_and_bad_suggestions(
            "send a notification every 30 minutes, starting in 2 hours from now",
            InTimeFromNow::default(),
            &[
                "send a notification every 30 minutes, starting 2 hours from now",
                "send a notification every 30 minutes, starting in 2 hours",
            ],
            &[],
        );
    }

    #[test]
    fn in_two_weeks_from_now() {
        assert_good_and_bad_suggestions(
            "That problem will be solved in two weeks from now by Bintray people.",
            InTimeFromNow::default(),
            &[
                "That problem will be solved two weeks from now by Bintray people.",
                "That problem will be solved in two weeks by Bintray people.",
            ],
            &[],
        );
    }
}
