use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
    patterns::InflectionOfBe,
};

pub struct WaistWaste {
    expr: FirstMatchOf,
}

impl Default for WaistWaste {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::any_word()
                        .t_ws()
                        .then(InflectionOfBe::default())
                        .t_ws()
                        .then_word_seq(&["a", "waist", "of"]),
                ),
                Box::new(SequenceExpr::word_seq(&["waist", "of"]).t_ws().t_set(&[
                    "effort",
                    "money",
                    "resources",
                    "space",
                    "time",
                ])),
                Box::new(
                    SequenceExpr::word_set(&[
                        "complete",
                        "confusing",
                        "great",
                        "horrible",
                        "huge",
                        "real",
                        "total",
                        "utter",
                    ])
                    .t_ws()
                    .then_word_seq(&["waist", "of"]),
                ),
                Box::new(
                    SequenceExpr::word_seq(&["a", "waist", "of"])
                        .t_ws()
                        .then_possessive_determiner(),
                ),
                Box::new(
                    SequenceExpr::word_set(&[
                        "definitely",
                        "just",
                        "mostly",
                        "probably",
                        "quite",
                        "really",
                    ])
                    .t_ws()
                    .then_word_seq(&["a", "waist", "of"]),
                ),
            ]),
        }
    }
}

impl ExprLinter for WaistWaste {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span =
            find_the_only_token_matching(toks, src, |t, s| t.get_ch(s).eq_str("waist"))?.span;

        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "waste",
            span.get_content(src),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message: "Did you mean `waste` (careless use) rather than `waist` (body part)?"
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects misspelling `waste` (careless use) as `waist` (body part)."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::WaistWaste;

    #[test]
    fn it_is_a_waist_of_space() {
        assert_suggestion_result(
            "The problem is, I think it is a waist of space pushing to remote all the software just because I have changed some files.",
            WaistWaste::default(),
            "The problem is, I think it is a waste of space pushing to remote all the software just because I have changed some files.",
        );
    }

    #[test]
    fn this_is_a_waist_of() {
        assert_suggestion_result(
            "This is a waist of Bs resources.",
            WaistWaste::default(),
            "This is a waste of Bs resources.",
        );
    }

    #[test]
    fn is_a_waist_of_space() {
        assert_suggestion_result(
            "The whole height takes only 19 rows, I think this is a waist of space",
            WaistWaste::default(),
            "The whole height takes only 19 rows, I think this is a waste of space",
        );
    }

    #[test]
    fn is_a_waist_of_time() {
        assert_suggestion_result(
            "Digging a hole with a toothpick is a waist of time.",
            WaistWaste::default(),
            "Digging a hole with a toothpick is a waste of time.",
        );
    }

    #[test]
    fn dont_flag_at_start() {
        assert_no_lints(
            "Is a waist of 46 inches around the belly button unhealthy??",
            WaistWaste::default(),
        );
    }

    #[test]
    fn waist_of_money() {
        assert_suggestion_result(
            "Copilot ==Waist of money. It's full of bugs",
            WaistWaste::default(),
            "Copilot ==Waste of money. It's full of bugs",
        );
    }

    #[test]
    fn huge_waist_of() {
        assert_suggestion_result(
            "A huge waist of time. If the goal cannot be fullfulled, the model will not fallback to debugging",
            WaistWaste::default(),
            "A huge waste of time. If the goal cannot be fullfulled, the model will not fallback to debugging",
        );
    }

    #[test]
    fn complete_waist() {
        assert_suggestion_result(
            "but what a complete waist of time to be stuck in a job for 8 hours a day",
            WaistWaste::default(),
            "but what a complete waste of time to be stuck in a job for 8 hours a day",
        );
    }

    #[test]
    fn total_waist() {
        assert_suggestion_result(
            "Total Waist Of Money & Time !! NOT RECOMMENDED !!!",
            WaistWaste::default(),
            "Total Waste Of Money & Time !! NOT RECOMMENDED !!!",
        );
    }

    #[test]
    fn a_waist_of_your_time() {
        assert_suggestion_result(
            "Nevertheless I learned some things form your answer so it wasn't a waist of your time.",
            WaistWaste::default(),
            "Nevertheless I learned some things form your answer so it wasn't a waste of your time.",
        );
    }

    #[test]
    fn just_a_waist_of() {
        assert_suggestion_result(
            "The \"temporary\" bitmap is just a waist of CPU resources.",
            WaistWaste::default(),
            "The \"temporary\" bitmap is just a waste of CPU resources.",
        );
    }
}
