use crate::{
    Lint, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct AsToInterrogative {
    expr: All,
}

impl Default for AsToInterrogative {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["curious", "unsure"])
                .t_ws()
                .t_aco("to")
                .t_ws()
                .t_set(&[
                    "how", "what", "what's", "which", "when", "when's", "where", "whether", "who",
                    "who's", "whose", "whom", "why",
                ])
                .and_not(
                    SequenceExpr::anything()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_ws()
                        // "Extend" would be a mistake here, but a common and unrelated mistake.
                        .t_set(&["degree", "extent", "extend"]),
                ),
        }
    }
}

impl ExprLinter for AsToInterrogative {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        if matched_tokens.len() < 5 {
            return None;
        }

        Some(Lint {
            span: matched_tokens[2].span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "as to",
                matched_tokens[2].get_ch(source),
            )],
            message: "This construction requires `as to` instead of just `to`.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `to` to `as to` between certain adjectives and `wh-words`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AsToInterrogative;

    #[test]
    fn fix_curious_how() {
        assert_suggestion_result(
            "I would be curious to how you dealt with the issues I mentioned above",
            AsToInterrogative::default(),
            "I would be curious as to how you dealt with the issues I mentioned above",
        );
    }

    #[test]
    fn fix_curious_what() {
        assert_suggestion_result(
            "I'm curious to what @Katzmann1983 thinks on this topic.",
            AsToInterrogative::default(),
            "I'm curious as to what @Katzmann1983 thinks on this topic.",
        );
    }

    #[test]
    fn dont_flag_curious_what_extent() {
        assert_no_lints(
            "I really just want to ask these questions / am curious to what extent anything like this is in place or will be put in place.",
            AsToInterrogative::default(),
        );
    }

    #[test]
    fn fix_curious_when() {
        assert_suggestion_result(
            "So I am curious to when you should return an array to destructure",
            AsToInterrogative::default(),
            "So I am curious as to when you should return an array to destructure",
        );
    }

    #[test]
    fn fix_curious_where() {
        assert_suggestion_result(
            "Curious to where you ended up.",
            AsToInterrogative::default(),
            "Curious as to where you ended up.",
        );
    }

    #[test]
    fn fix_curious_whether() {
        assert_suggestion_result(
            "I have never used them so was curious to whether I could improve my workflow even more",
            AsToInterrogative::default(),
            "I have never used them so was curious as to whether I could improve my workflow even more",
        );
    }

    #[test]
    fn fix_curious_which() {
        assert_suggestion_result(
            "I'm curious to which country you're assigning these noble ideals.",
            AsToInterrogative::default(),
            "I'm curious as to which country you're assigning these noble ideals.",
        );
    }

    #[test]
    fn fix_curious_who() {
        assert_suggestion_result(
            "New to climbing, curious to who create the problems/leads in the Olympics and how/by who are they tested?",
            AsToInterrogative::default(),
            "New to climbing, curious as to who create the problems/leads in the Olympics and how/by who are they tested?",
        );
    }

    #[test]
    fn fix_curious_why() {
        assert_suggestion_result(
            "I am curious to why the below weird_query does not yield a query with the parent relationship loaded",
            AsToInterrogative::default(),
            "I am curious as to why the below weird_query does not yield a query with the parent relationship loaded",
        );
    }

    #[test]
    fn fix_unsure_what() {
        assert_suggestion_result(
            "If you are unsure to what BurpSuite is, or how to set it up please complete our BurpSuite room first.",
            AsToInterrogative::default(),
            "If you are unsure as to what BurpSuite is, or how to set it up please complete our BurpSuite room first.",
        );
    }

    #[test]
    fn dont_flag_unsure_what_degree() {
        assert_no_lints(
            "and I'm unsure to what degree these have already been done",
            AsToInterrogative::default(),
        );
    }

    #[test]
    #[ignore = "This edge case with 'go' is tricky to handle"]
    fn dont_flag_unsure_which_go() {
        assert_no_lints(
            "Two classical music performances on the same day, unsure to which I should go.",
            AsToInterrogative::default(),
        );
    }

    #[test]
    fn fix_unsure_why() {
        assert_suggestion_result(
            "I am unsure to why this code is wrong.",
            AsToInterrogative::default(),
            "I am unsure as to why this code is wrong.",
        );
    }
}
