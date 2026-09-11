use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

pub struct SummarySummery {
    expr: FirstMatchOf,
}

impl Default for SummarySummery {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::aco("summary")
                        .t_ws()
                        .t_set(["dress", "dresses"]),
                ),
                Box::new(
                    SequenceExpr::word_set(["executive", "financial"])
                        .t_ws()
                        .t_aco("summery"),
                ),
                Box::new(SequenceExpr::aco("summery").t_ws().t_set([
                    "about", "above", "below", "by", "for", "in", "into", "log", "of", "to", "with",
                ])),
            ]),
        }
    }
}

impl ExprLinter for SummarySummery {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = find_the_only_token_matching(toks, src, |t, s| {
            t.get_ch(s)
                .eq_any_ignore_ascii_case_str(&["summary", "summery"])
        })?
        .span;

        let mut correction: Vec<char> = span.get_content(src).to_vec();

        let swapped_a_e = match correction.get(4)? {
            'a' => 'e',
            'A' => 'E',
            'e' => 'a',
            'E' => 'A',
            _ => return None,
        };

        correction[4] = swapped_a_e;

        let suggestions = vec![Suggestion::ReplaceWith(correction)];

        Some(Lint {
            span,
            lint_kind: LintKind::Spelling,
            suggestions,
            message: "Did you mean `summery` (relating to summer) or `summary` (brief statement)?"
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Checks for the common confusion between `summary` and `summery`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::SummarySummery;

    // Summary should be Summery

    #[test]
    fn summary_dress() {
        assert_suggestion_result(
            "Beautiful summary dress product code mf 78.",
            SummarySummery::default(),
            "Beautiful summery dress product code mf 78.",
        );
    }

    // Summery should be Summary

    #[test]
    fn executive_summery() {
        assert_suggestion_result(
            "I will also post a sample executive summery found online.",
            SummarySummery::default(),
            "I will also post a sample executive summary found online.",
        );
    }

    #[test]
    fn financial_summery() {
        assert_suggestion_result(
            "BOQs Financial Summery Application ADE 5838.",
            SummarySummery::default(),
            "BOQs Financial Summary Application ADE 5838.",
        );
    }

    #[test]
    fn summery_about() {
        assert_suggestion_result(
            "Again, thanks for the nice summery about the fix.",
            SummarySummery::default(),
            "Again, thanks for the nice summary about the fix.",
        );
    }

    #[test]
    fn summery_above() {
        assert_suggestion_result(
            "One issue I can think of immediately while reading @gautelund's summery above is with the proposal",
            SummarySummery::default(),
            "One issue I can think of immediately while reading @gautelund's summary above is with the proposal",
        );
    }

    #[test]
    fn summery_below() {
        assert_suggestion_result(
            "Report Summery Below",
            SummarySummery::default(),
            "Report Summary Below",
        );
    }

    #[test]
    fn summery_by() {
        assert_suggestion_result(
            "node has crashed my system hard here is a summery by claude",
            SummarySummery::default(),
            "node has crashed my system hard here is a summary by claude",
        );
    }

    #[test]
    fn summery_for() {
        assert_suggestion_result(
            "A tool that trys to make condensed summery for Bugzilla bugs",
            SummarySummery::default(),
            "A tool that trys to make condensed summary for Bugzilla bugs",
        );
    }

    #[test]
    fn summery_in() {
        assert_suggestion_result(
            "Scrapes and generates a descriptive summery in pdf from any url",
            SummarySummery::default(),
            "Scrapes and generates a descriptive summary in pdf from any url",
        );
    }

    #[test]
    fn summery_into() {
        assert_suggestion_result(
            "It would be awesome if you can put your summery into API documentation",
            SummarySummery::default(),
            "It would be awesome if you can put your summary into API documentation",
        );
    }

    #[test]
    fn summery_log() {
        assert_suggestion_result(
            "Here is summery log: summary.log. Any advise please?",
            SummarySummery::default(),
            "Here is summary log: summary.log. Any advise please?",
        );
    }

    #[test]
    fn summery_of() {
        assert_suggestion_result(
            "Summery of the contamination and progress.",
            SummarySummery::default(),
            "Summary of the contamination and progress.",
        );
    }

    #[test]
    fn summery_to() {
        assert_suggestion_result(
            "add archive summery to the console output/log.",
            SummarySummery::default(),
            "add archive summary to the console output/log.",
        );
    }

    #[test]
    fn summery_with() {
        assert_suggestion_result(
            "Yet not code-coverage summery with coverage percentage and missing lines",
            SummarySummery::default(),
            "Yet not code-coverage summary with coverage percentage and missing lines",
        );
    }
}
