use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr, SpelledNumberExpr, TimeUnitExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct GoBackUnitsAgo {
    expr: SequenceExpr,
}

impl Default for GoBackUnitsAgo {
    fn default() -> Self {
        let quantity = SequenceExpr::longest_of(vec![
            Box::new(SpelledNumberExpr),
            Box::new(SequenceExpr::default().then_number()),
            Box::new(SequenceExpr::aco("a").t_ws().then_quantifier()),
            Box::new(SequenceExpr::default().then_indefinite_article()),
            Box::new(SequenceExpr::default().then_quantifier()),
        ]);

        Self {
            expr: SequenceExpr::word_set(&["go", "goes", "going", "gone", "went"])
                .t_ws()
                .t_aco("back")
                .t_ws()
                .then_optional(quantity.t_ws())
                .then(TimeUnitExpr::default())
                .t_ws()
                .t_aco("ago"),
        }
    }
}

impl ExprLinter for GoBackUnitsAgo {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], _source: &[char]) -> Option<Lint> {
        let ago_with_space = toks[toks.len().saturating_sub(2)..].span()?;

        Some(Lint {
            span: ago_with_space,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::Remove],
            message: "`Go back` already expresses a span of time, so `ago` is redundant here."
                .to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Flags redundant `ago` after `go back` followed by a time expression."
    }
}

#[cfg(test)]
mod tests {
    use super::GoBackUnitsAgo;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn fixes_goes_back_40_years_ago() {
        assert_suggestion_result(
            "This theory goes back 40 years ago.",
            GoBackUnitsAgo::default(),
            "This theory goes back 40 years.",
        );
    }

    #[test]
    fn flags_happy_path_once() {
        assert_lint_count(
            "This theory goes back 40 years ago.",
            GoBackUnitsAgo::default(),
            1,
        );
    }

    #[test]
    fn fixes_go_back_20_years_ago() {
        assert_suggestion_result(
            "I can go back 20 years ago in the archive.",
            GoBackUnitsAgo::default(),
            "I can go back 20 years in the archive.",
        );
    }

    #[test]
    fn fixes_going_back_hours_ago() {
        assert_suggestion_result(
            "Going back hours ago, the logs were already noisy.",
            GoBackUnitsAgo::default(),
            "Going back hours, the logs were already noisy.",
        );
    }

    #[test]
    fn fixes_gone_back_months_ago() {
        assert_suggestion_result(
            "The notes have gone back months ago.",
            GoBackUnitsAgo::default(),
            "The notes have gone back months.",
        );
    }

    #[test]
    fn fixes_went_back_two_days_ago() {
        assert_suggestion_result(
            "We went back two days ago to compare the results.",
            GoBackUnitsAgo::default(),
            "We went back two days to compare the results.",
        );
    }

    #[test]
    fn fixes_hyphenated_spelled_number() {
        assert_suggestion_result(
            "The record went back twenty-one years ago.",
            GoBackUnitsAgo::default(),
            "The record went back twenty-one years.",
        );
    }

    #[test]
    fn fixes_article_quantity() {
        assert_suggestion_result(
            "The backup goes back a year ago.",
            GoBackUnitsAgo::default(),
            "The backup goes back a year.",
        );
    }

    #[test]
    fn fixes_a_few_quantity() {
        assert_suggestion_result(
            "The trail goes back a few weeks ago.",
            GoBackUnitsAgo::default(),
            "The trail goes back a few weeks.",
        );
    }

    #[test]
    fn fixes_uppercase_verb() {
        assert_suggestion_result(
            "GO back 5 minutes ago and read it again.",
            GoBackUnitsAgo::default(),
            "GO back 5 minutes and read it again.",
        );
    }

    #[test]
    fn fixes_abbreviated_unit() {
        assert_suggestion_result(
            "The snapshot went back 20 ms ago.",
            GoBackUnitsAgo::default(),
            "The snapshot went back 20 ms.",
        );
    }

    #[test]
    fn doesnt_flag_go_back_to_years_ago() {
        assert_no_lints(
            "This theory goes back to 40 years ago.",
            GoBackUnitsAgo::default(),
        );
    }

    #[test]
    fn doesnt_flag_dates_back() {
        assert_no_lints("This dates back 40 years.", GoBackUnitsAgo::default());
    }

    #[test]
    fn doesnt_flag_without_ago() {
        assert_no_lints("This theory goes back 40 years.", GoBackUnitsAgo::default());
    }

    #[test]
    fn doesnt_flag_later() {
        assert_no_lints("I went back 20 years later.", GoBackUnitsAgo::default());
    }

    #[test]
    fn doesnt_flag_unrelated_back() {
        assert_no_lints(
            "I went back there years ago to check the records.",
            GoBackUnitsAgo::default(),
        );
    }
}
