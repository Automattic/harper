use crate::{
    CharStringExt, Lint, Token,
    expr::{AnchorEnd, Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

pub struct HighLowStandards {
    expr: SequenceExpr,
}

impl Default for HighLowStandards {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["got", "had", "has", "have", "having"])
                .then_optional(SequenceExpr::whitespace().then_degree_adverb())
                .t_ws()
                .t_set(["high", "low"])
                .t_ws()
                .t_aco("standard")
                .then_any_of([
                    Box::new(AnchorEnd) as Box<dyn Expr>,
                    Box::new(SequenceExpr::whitespace().then(|t: &Token, s: &[char]| {
                        (t.kind.is_preposition() && !t.get_ch(s).eq_str("of"))
                            || t.get_ch(s)
                                .eq_any_ignore_ascii_case_str(&["and", "even", "when"])
                    })),
                ]),
        }
    }
}

impl ExprLinter for HighLowStandards {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span =
            find_the_only_token_matching(toks, src, |t, _| t.get_ch(src).eq_str("standard"))?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "standards",
                span.get_content(src),
            )],
            message: "This idiom uses plural `standards`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `high/low standard` to `high/low standards` when `standard` is a noun."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::HighLowStandards;

    #[test]
    fn at_end() {
        assert_suggestion_result(
            "I have low standard",
            HighLowStandards::default(),
            "I have low standards",
        );
    }

    #[test]
    fn followed_by_for() {
        assert_suggestion_result(
            "I have high standard for men",
            HighLowStandards::default(),
            "I have high standards for men",
        );
    }

    #[test]
    fn dont_flag_standard_deviation() {
        assert_no_lints(
            "It's very difficult for any model to predict accurately when trained with data having such high standard deviation.",
            HighLowStandards::default(),
        );
    }

    #[test]
    fn dont_flag_followed_by_when() {
        assert_suggestion_result(
            "I tend to have high standard when answering questions.",
            HighLowStandards::default(),
            "I tend to have high standards when answering questions.",
        );
    }

    #[test]
    fn followed_by_comma() {
        assert_suggestion_result(
            "Most student would choose supervisor who doesn’t have high standard, but what i did is precisely the opposite.",
            HighLowStandards::default(),
            "Most student would choose supervisor who doesn’t have high standards, but what i did is precisely the opposite.",
        );
    }

    #[test]
    fn very_high() {
        assert_suggestion_result(
            "I picked a supervisor who has very high standard, strict, and very well detailed.",
            HighLowStandards::default(),
            "I picked a supervisor who has very high standards, strict, and very well detailed.",
        );
    }

    #[test]
    fn followed_by_and() {
        assert_suggestion_result(
            "Great way to tell you have low standard and can't detect latency.",
            HighLowStandards::default(),
            "Great way to tell you have low standards and can't detect latency.",
        );
    }

    #[test]
    fn dont_flag_standard_of_living() {
        assert_no_lints(
            "US has high standard of living, but it is not among the top 5 countreis with the highest minimum hourly-wage.",
            HighLowStandards::default(),
        );
    }

    #[test]
    fn followed_by_on() {
        assert_suggestion_result(
            "We should also recognize the fact that, Swedes have very high standard on their expectation of English fluency.",
            HighLowStandards::default(),
            "We should also recognize the fact that, Swedes have very high standards on their expectation of English fluency.",
        );
    }

    #[test]
    fn followed_by_in() {
        assert_suggestion_result(
            "We have high standard in product manufacturing and foreign trade",
            HighLowStandards::default(),
            "We have high standards in product manufacturing and foreign trade",
        );
    }
}
