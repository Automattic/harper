use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

pub struct WeSeeUs {
    expr: SequenceExpr,
}

impl Default for WeSeeUs {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("we")
                .t_ws()
                .then_any_of([
                    Box::new(WordSet::new(["see", "saw"])) as Box<dyn Expr>,
                    Box::new(
                        SequenceExpr::word_set([
                            "can",
                            "could",
                            "may",
                            "might",
                            "must",
                            "shall",
                            "should",
                            "will",
                            "would",
                            "can't",
                            "couldn't",
                            "mightn't",
                            "mustn't",
                            "shan't",
                            "shouldn't",
                            "won't",
                            "wouldn't",
                            "cannot",
                        ])
                        .t_ws()
                        .t_aco("see"),
                    ),
                    Box::new(SequenceExpr::word_set(["have", "had"]).t_ws().t_aco("seen")),
                    Box::new(SequenceExpr::word_seq(&["are", "seeing"])),
                ])
                .t_ws()
                .t_aco("us"),
        }
    }
}

impl ExprLinter for WeSeeUs {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        // "us" and "Us" look like the pronoun, but "US" looks like the country
        let us_tok = matched_tokens.last()?;
        let us_ch = us_tok.get_ch(source);
        if us_ch == ['U', 'S'] {
            return None;
        }
        let span = us_tok.span;
        let lint_kind = LintKind::Miscellaneous;
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "each other",
            span.get_content(source),
        )];
        let message = "Did you mean `each other`?".to_owned();
        Some(Lint {
            span,
            lint_kind,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Looks for the common Germanic phrase `we see us` and suggests `we see each other` as a replacement."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::WeSeeUs;

    #[test]
    fn see() {
        assert_suggestion_result(
            "In the example below we see us using a schema to validate a configuration and setup defaults.",
            WeSeeUs::default(),
            "In the example below we see each other using a schema to validate a configuration and setup defaults.",
        );
    }

    #[test]
    fn saw() {
        assert_suggestion_result(
            "From time to time we saw us in front of meaningless stack traces.",
            WeSeeUs::default(),
            "From time to time we saw each other in front of meaningless stack traces.",
        )
    }

    #[test]
    fn can_see() {
        assert_suggestion_result(
            "We can see us chucking the nodes or in this case, we most likely have to give it back to them",
            WeSeeUs::default(),
            "We can see each other chucking the nodes or in this case, we most likely have to give it back to them",
        )
    }

    #[test]
    fn cant_see() {
        assert_suggestion_result(
            "On the list endpoints, we can't see us ever leaving the older (current) endpoints behind",
            WeSeeUs::default(),
            "On the list endpoints, we can't see each other ever leaving the older (current) endpoints behind",
        )
    }

    #[test]
    fn should_see() {
        assert_suggestion_result(
            "If we can see shadows on the ground, we should see us making shadows.",
            WeSeeUs::default(),
            "If we can see shadows on the ground, we should see each other making shadows.",
        )
    }

    #[test]
    fn will_see() {
        assert_suggestion_result(
            "we will see us through others and we will connect our experiences with testimonies of people",
            WeSeeUs::default(),
            "we will see each other through others and we will connect our experiences with testimonies of people",
        );
    }

    #[test]
    fn have_seen() {
        assert_suggestion_result(
            "We have seen us almost every Day for the last 5 years.",
            WeSeeUs::default(),
            "We have seen each other almost every Day for the last 5 years.",
        )
    }

    #[test]
    fn had_seen() {
        assert_suggestion_result(
            "We are very lucky about the fact that we had seen us in 2009, while we were spending our vacation with you.",
            WeSeeUs::default(),
            "We are very lucky about the fact that we had seen each other in 2009, while we were spending our vacation with you.",
        )
    }

    #[test]
    fn are_seeing() {
        assert_suggestion_result(
            "We are seeing us through their eyes.",
            WeSeeUs::default(),
            "We are seeing each other through their eyes.",
        )
    }

    #[test]
    fn dont_flag_see_us_caps() {
        assert_no_lints(
            "From the array, above we see US customers are listed as \"USA\"",
            WeSeeUs::default(),
        )
    }

    #[test]
    fn dont_flag_can_see_us_caps() {
        assert_no_lints(
            "We can see US is the largest contributor of netflix",
            WeSeeUs::default(),
        )
    }
}
