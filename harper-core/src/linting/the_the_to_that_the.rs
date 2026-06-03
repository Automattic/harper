use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct TheTheToThatThe {
    expr: SequenceExpr,
}

impl Default for TheTheToThatThe {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&[
                "so", "fact", "found", "show", "say", "believe", "think", "said",
            ])
            .t_ws()
            .t_aco("the")
            .t_ws()
            .t_aco("the"),
        }
    }
}

impl ExprLinter for TheTheToThatThe {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks[toks
            .iter()
            .position(|t| t.get_ch(src).eq_ch(&['t', 'h', 'e']))?..][..3]
            .span()?;
        let ch = span.get_content(src);

        Some(Lint {
            span,
            lint_kind: LintKind::Typo,
            suggestions: ["that the", "the"]
                .iter()
                .map(|s| Suggestion::replace_with_match_case_str(s, ch))
                .collect(),
            message: "Did you mean `that the` or just `the`?".to_string(),
            priority: 126, // Higher priority than `RepeatedWords`
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `the the` to `that the` or to a single `the`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
    };

    use super::TheTheToThatThe;

    // Contrived tests

    #[test]
    fn dont_flag_simple_the_the() {
        assert_no_lints("this is the the simple case", TheTheToThatThe::default());
    }

    #[test]
    fn fix_so_that_the() {
        assert_suggestion_result(
            "this is so the the other case is tested",
            TheTheToThatThe::default(),
            "this is so that the other case is tested",
        );
    }

    // Real world tests

    #[test]
    fn fact_the_the_to_that() {
        assert_suggestion_result(
            "This PR fixes the fact the the / page and some of the other new pages are reverting to the browser font",
            TheTheToThatThe::default(),
            "This PR fixes the fact that the / page and some of the other new pages are reverting to the browser font",
        );
    }

    #[test]
    fn found_the_the_just_double() {
        assert_suggestion_result(
            "and I've never found the the dashboard feed materially useful",
            TheTheToThatThe::default(),
            "and I've never found the dashboard feed materially useful",
        );
    }

    #[test]
    fn found_the_the_could_be_either() {
        assert_good_and_bad_suggestions(
            "I found the the shape of feature from the PillarFeatureNet becomes [64] instead of [N, 64]",
            TheTheToThatThe::default(),
            &[
                "I found that the shape of feature from the PillarFeatureNet becomes [64] instead of [N, 64]",
                "I found the shape of feature from the PillarFeatureNet becomes [64] instead of [N, 64]",
            ],
            &[],
        );
    }

    #[test]
    fn found_the_the_to_that() {
        assert_suggestion_result(
            "I was directed to the MongoDB website and found the the newer versions require you to have the mongosh shell",
            TheTheToThatThe::default(),
            "I was directed to the MongoDB website and found that the newer versions require you to have the mongosh shell",
        );
    }

    #[test]
    fn say_the_the_to_that() {
        assert_suggestion_result(
            "So, I'm going to say the the shown output is a bug in using Avro",
            TheTheToThatThe::default(),
            "So, I'm going to say that the shown output is a bug in using Avro",
        );
    }

    #[test]
    fn show_the_the_just_double() {
        assert_suggestion_result(
            "It would be better to show the the first logical line (i.e. the first paragraph).",
            TheTheToThatThe::default(),
            "It would be better to show the first logical line (i.e. the first paragraph).",
        );
    }

    #[test]
    fn so_the_the_could_be_either() {
        assert_good_and_bad_suggestions(
            "accordion-body doesn't have data- attributes to read from, so the The proper solution isn't quite working",
            TheTheToThatThe::default(),
            &[
                // TODO: replace_with_match_cases goes by index so results in `thaT` instead of `that`
                // "accordion-body doesn't have data- attributes to read from, so that the proper solution isn't quite working",
                "accordion-body doesn't have data- attributes to read from, so the proper solution isn't quite working",
            ],
            &[],
        );
    }

    #[test]
    fn so_the_the_just_double() {
        assert_suggestion_result(
            "So the the choice is really between pursuing a compromise ...",
            TheTheToThatThe::default(),
            "So the choice is really between pursuing a compromise ...",
        );
    }

    #[test]
    fn think_the_the_to_that() {
        assert_suggestion_result(
            "I don't think the the additional flask route was the issue because I got your example to work.",
            TheTheToThatThe::default(),
            "I don't think that the additional flask route was the issue because I got your example to work.",
        );
    }

    #[test]
    fn think_the_the_could_be_either() {
        assert_good_and_bad_suggestions(
            "but I think the the lack of transparency and attribution to the original coder makes this much more an act of opportunism",
            TheTheToThatThe::default(),
            &[
                "but I think that the lack of transparency and attribution to the original coder makes this much more an act of opportunism",
                "but I think the lack of transparency and attribution to the original coder makes this much more an act of opportunism",
            ],
            &[],
        );
    }
}
