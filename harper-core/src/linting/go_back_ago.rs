use crate::Token;
use crate::expr::{DurationExpr, Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};
use crate::token_string_ext::TokenStringExt;

/// Detects the redundant construction "go back [duration] ago".
///
/// "Go back" already implies retrospection, so adding "ago" is redundant.
/// For example, "goes back 40 years ago" should be "goes back 40 years".
pub struct GoBackAgo {
    expr: SequenceExpr,
}

impl Default for GoBackAgo {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["go", "goes", "going", "gone", "went"])
                .t_ws()
                .t_aco("back")
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
                .t_aco("ago"),
        }
    }
}

impl ExprLinter for GoBackAgo {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // Find the index of "back" and "ago" in the matched tokens
        let back_idx = toks
            .iter()
            .position(|t| t.kind.is_word() && t.get_str(src).eq_ignore_ascii_case("back"))?;

        let ago_idx = toks
            .iter()
            .rposition(|t| t.kind.is_word() && t.get_str(src).eq_ignore_ascii_case("ago"))?;

        // Suggestion 1: Remove "ago" and the preceding whitespace.
        // "goes back 40 years ago" → "goes back 40 years"
        // toks[ago_idx - 1] is the whitespace before "ago"
        let without_ago: Vec<char> = toks[..ago_idx - 1].span()?.get_content(src).to_vec();

        // Suggestion 2: Remove "back" and its surrounding whitespace.
        // "goes back 40 years ago" → "goes 40 years ago"
        // We compose: verb + ws_after_verb + everything_from_after_back_ws_to_end
        let without_back: Vec<char> = {
            let verb_end = toks[0].span.end;
            // back_idx + 1 is the whitespace after "back"
            let after_back_ws_end = toks[back_idx + 1].span.end;
            let match_end = toks.last()?.span.end;

            let mut result = Vec::new();
            // Verb
            result.extend_from_slice(&src[toks[0].span.start..verb_end]);
            // Whitespace between verb and what follows after "back"
            result.extend_from_slice(&src[verb_end..toks[1].span.end]);
            // Everything from after "back"+ws
            result.extend_from_slice(&src[after_back_ws_end..match_end]);
            result
        };

        let template_chars = toks.span()?.get_content(src);

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Redundancy,
            message: "`back` and `ago` are redundant together. Use one or the other.".to_string(),
            suggestions: vec![
                Suggestion::replace_with_match_case(without_ago, template_chars),
                Suggestion::replace_with_match_case(without_back, template_chars),
            ],
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Checks for the word `ago` following `go back [a period of time]`, which is redundant. Does not flag `go back to [duration] ago`, where `to` introduces a point in time and `ago` is not redundant."
    }
}

#[cfg(test)]
mod tests {
    use super::GoBackAgo;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    // --- True positives: "go back [duration] ago" should be flagged ---

    #[test]
    fn goes_back_40_years_ago() {
        assert_suggestion_result(
            "And this theory goes back 40 years ago to a deal.",
            GoBackAgo::default(),
            "And this theory goes back 40 years to a deal.",
        );
    }

    #[test]
    fn go_back_20_years_ago() {
        assert_suggestion_result(
            "You have to go back 20 years ago to find a Prize.",
            GoBackAgo::default(),
            "You have to go back 20 years to find a Prize.",
        );
    }

    #[test]
    fn going_back_ten_years_ago() {
        assert_suggestion_result(
            "Going back ten years ago, the landscape was very different.",
            GoBackAgo::default(),
            "Going back ten years, the landscape was very different.",
        );
    }

    #[test]
    fn went_back_five_minutes_ago() {
        assert_suggestion_result(
            "He went back five minutes ago to check the results.",
            GoBackAgo::default(),
            "He went back five minutes to check the results.",
        );
    }

    #[test]
    fn gone_back_a_year_ago() {
        assert_suggestion_result(
            "They have gone back a year ago in the historical records.",
            GoBackAgo::default(),
            "They have gone back a year in the historical records.",
        );
    }

    #[test]
    fn goes_back_about_three_years_ago() {
        assert_suggestion_result(
            "The tradition goes back about three years ago.",
            GoBackAgo::default(),
            "The tradition goes back about three years.",
        );
    }

    #[test]
    fn go_back_just_five_minutes_ago() {
        assert_suggestion_result(
            "I had to go back just five minutes ago to find the bug.",
            GoBackAgo::default(),
            "I had to go back just five minutes to find the bug.",
        );
    }

    #[test]
    fn goes_back_to_40_years_ago_is_valid() {
        assert_no_lints("This goes back to 40 years ago.", GoBackAgo::default());
    }

    #[test]
    fn going_back_to_ten_years_ago_is_valid() {
        assert_no_lints(
            "We're going back to ten years ago to find the origin.",
            GoBackAgo::default(),
        );
    }

    #[test]
    fn go_back_to_a_week_ago_is_valid() {
        assert_no_lints(
            "Let's go back to a week ago and see what happened.",
            GoBackAgo::default(),
        );
    }

    #[test]
    fn go_back_almost_ten_years_ago() {
        assert_suggestion_result(
            "This goes back almost ten years ago.",
            GoBackAgo::default(),
            "This goes back almost ten years.",
        );
    }

    #[test]
    fn went_back_to_five_days_ago_is_valid() {
        assert_no_lints(
            "He went back to five days ago in the version history.",
            GoBackAgo::default(),
        );
    }

    // --- Alternative suggestion: remove "back" ---

    #[test]
    fn goes_back_years_ago_remove_back() {
        assert_suggestion_result(
            "This goes back 40 years ago.",
            GoBackAgo::default(),
            "This goes 40 years ago.",
        );
    }

    // --- True negatives: no redundancy, should not be flagged ---

    #[test]
    fn dont_flag_goes_back_40_years() {
        assert_no_lints(
            "And this theory goes back 40 years to a deal.",
            GoBackAgo::default(),
        );
    }

    #[test]
    fn dont_flag_go_back_home() {
        assert_no_lints("I want to go back home now.", GoBackAgo::default());
    }

    #[test]
    fn dont_flag_years_ago_without_go_back() {
        assert_no_lints("This happened 40 years ago.", GoBackAgo::default());
    }

    #[test]
    fn dont_flag_go_back() {
        assert_no_lints("Please go back to the previous page.", GoBackAgo::default());
    }

    #[test]
    fn dont_flag_five_minutes_ago() {
        assert_no_lints(
            "He checked the results five minutes ago.",
            GoBackAgo::default(),
        );
    }

    #[test]
    fn dont_flag_went_back_to_sleep() {
        assert_no_lints(
            "She went back to sleep after the alarm.",
            GoBackAgo::default(),
        );
    }

    #[test]
    fn dont_flag_back_then() {
        assert_no_lints("Back then, things were different.", GoBackAgo::default());
    }

    #[test]
    fn dont_flag_go_back_and_check() {
        assert_no_lints(
            "Go back and check the logs from a year ago.",
            GoBackAgo::default(),
        );
    }

    #[test]
    fn dont_flag_way_back() {
        assert_no_lints("The way back home took an hour.", GoBackAgo::default());
    }

    #[test]
    fn dont_flag_dont_go_back() {
        assert_no_lints("Don't go back there.", GoBackAgo::default());
    }

    #[test]
    fn dont_flag_go_back_two_years() {
        assert_no_lints(
            "We need to go back two years to find the origin.",
            GoBackAgo::default(),
        );
    }
}
