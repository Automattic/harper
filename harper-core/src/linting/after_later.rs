use crate::Token;
use crate::expr::{DurationExpr, Expr, SequenceExpr};
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};
use crate::token_string_ext::TokenStringExt;

pub struct AfterLater {
    expr: Box<dyn Expr>,
}

impl Default for AfterLater {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::aco("after")
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
                    .t_aco("later"),
            ),
        }
    }
}

impl ExprLinter for AfterLater {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let without_after: Vec<char> = toks[2..].span()?.get_content(src).to_vec();
        let without_later: Vec<char> = toks[..toks.len() - 2].span()?.get_content(src).to_vec();

        let template_chars = toks.span()?.get_content(src);

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Redundancy,
            message: "Don't use `later` following `after [a period of time]`".to_string(),
            suggestions: vec![
                Suggestion::replace_with_match_case(without_after, template_chars),
                Suggestion::replace_with_match_case(without_later, template_chars),
            ],
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Checks for the word `later` following `after [a period of time]`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{AfterLater, tests::assert_top3_suggestion_result};

    #[test]
    fn after_90_days_later() {
        assert_top3_suggestion_result(
            "Try to rename your organization after 90 days later because of GitHub official documentation it said.",
            AfterLater::default(),
            "Try to rename your organization after 90 days because of GitHub official documentation it said.",
        );
    }

    #[test]
    fn after_about_30_minutes_later() {
        assert_top3_suggestion_result(
            "It plays like 1 minute of the song and then stops, and after about 30 minutes later, the bot disconnects an throws DisTubeError",
            AfterLater::default(),
            "It plays like 1 minute of the song and then stops, and about 30 minutes later, the bot disconnects an throws DisTubeError",
        );
    }

    #[test]
    fn after_14_days_later() {
        assert_top3_suggestion_result(
            "After 14 days later, the cache expired.",
            AfterLater::default(),
            "After 14 days, the cache expired.",
        );
    }

    #[test]
    fn after_exactly_5_minutes_later() {
        assert_top3_suggestion_result(
            "After exactly 5 minutes later, they try again and the cluster is formed then.",
            AfterLater::default(),
            "Exactly 5 minutes later, they try again and the cluster is formed then.",
        );
    }

    #[test]
    fn after_22_years_later_1() {
        assert_top3_suggestion_result(
            "Completed YR campaign for 2nd time after 22 years later.",
            AfterLater::default(),
            "Completed YR campaign for 2nd time after 22 years.",
        );
    }

    #[test]
    fn after_almost_2_years_later() {
        assert_top3_suggestion_result(
            "This buyer contacted me after almost 2 years later.",
            AfterLater::default(),
            "This buyer contacted me almost 2 years later.",
        );
    }

    #[test]
    fn after_2_years_later() {
        assert_top3_suggestion_result(
            "Is Jedi Survivor better now after 2 years later?",
            AfterLater::default(),
            "Is Jedi Survivor better now after 2 years?",
        );
    }

    #[test]
    fn after_a_year_later() {
        assert_top3_suggestion_result(
            "Even after a year later, I don’t know how to get my self-love back.",
            AfterLater::default(),
            "Even a year later, I don’t know how to get my self-love back.",
        );
    }

    #[test]
    fn after_22_years_later_2() {
        assert_top3_suggestion_result(
            "After 22 years later, my top 1 game was Zeroed",
            AfterLater::default(),
            "After 22 years, my top 1 game was Zeroed",
        );
    }
}
