use crate::{
    Lint, Token, TokenKind,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct InDemandInDepth {
    expr: SequenceExpr,
}

impl Default for InDemandInDepth {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_kind_any_but_not(
                    // To flag:
                    // ✔ [Get] in depth answers                            - verb lemma
                    // ✔ consisting [of] in depth technical discussion     - preposition
                    // ✔ [more]/[most]                                     - determiner/quantifier? adverb?
                    // Not to flag:
                    // ✘ explore these [concepts] in depth                 - plural noun
                    // ✘ [Defense] in Depth                                - abstract/mass noun
                    // ✘ and [MCP] in depth                                - mass noun
                    &[
                        TokenKind::is_verb_lemma,
                        TokenKind::is_preposition,
                        TokenKind::is_quantifier,
                        TokenKind::is_degree_adverb,
                    ],
                    TokenKind::is_mass_noun,
                )
                .t_ws()
                .t_aco("in")
                .t_ws()
                .t_set(&["demand", "demands", "depth", "depths"]),
        }
    }
}

impl ExprLinter for InDemandInDepth {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let Some(tok) = toks.get(3) else {
            return None;
        };

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Punctuation,
            suggestions: vec![Suggestion::ReplaceWith(vec!['-'])],
            message: "When used as adjectives, `in-demand` and `in-depth` should be hyphenated."
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Checks for `in-demand` and `in-depth` used as adjectives but not hyphenated."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::InDemandInDepth;

    #[test]
    fn of_in_depth_technical_discussion() {
        assert_suggestion_result(
            "One hour video call consisting of in depth technical discussion with lead developers",
            InDemandInDepth::default(),
            "One hour video call consisting of in-depth technical discussion with lead developers",
        );
    }

    #[test]
    fn a_more_in_depth_documentation() {
        assert_suggestion_result(
            "I have added a more in depth documentation for each concept",
            InDemandInDepth::default(),
            "I have added a more in-depth documentation for each concept",
        );
    }

    #[test]
    fn most_in_demand_job_technologies() {
        assert_suggestion_result(
            "Jupyter notebook for scraping and analysis of most in demand job technologies skills for data scientists",
            InDemandInDepth::default(),
            "Jupyter notebook for scraping and analysis of most in-demand job technologies skills for data scientists",
        );
    }

    #[test]
    fn invited_engineer_to_explain_more_in_depth() {
        assert_suggestion_result(
            "invited one of their engineers to the next meeting to explain this more in depth",
            InDemandInDepth::default(),
            "invited one of their engineers to the next meeting to explain this more in-depth",
        );
    }

    #[test]
    fn very_in_depth_explination() {
        assert_suggestion_result(
            "This is a very in depth explination of naive bayes w.r.t implementation in python",
            InDemandInDepth::default(),
            "This is a very in-depth explination of naive bayes w.r.t implementation in python",
        );
    }

    #[test]
    fn pretty_in_depth_comparison() {
        assert_suggestion_result(
            "there should be a pretty in depth comparison of all of the options in the wiki",
            InDemandInDepth::default(),
            "there should be a pretty in-depth comparison of all of the options in the wiki",
        );
    }

    #[test]
    fn quite_in_depth_analysis() {
        assert_suggestion_result(
            "There was already a quite in depth analysis for Tasmota here",
            InDemandInDepth::default(),
            "There was already a quite in-depth analysis for Tasmota here",
        );
    }
}
