use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

// Prepositions which work after "summary" but not after "summery". We can't use .then_preposition()
// since some prepositions work after both.
const POST_SUMMARY_PREPOSITIONS: &[&str] = &[
    "about", "above", "below", "by", "for", "in", "into", "of", "to", "with",
];

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
                Box::new(SequenceExpr::aco("summery").t_ws_h().t_set(
                    POST_SUMMARY_PREPOSITIONS.iter().copied().chain([
                        "box", "boxes", "div", "control", "controls", "element", "elements",
                        "field", "fields", "log", "logs", "object", "objects", "page", "pages",
                        "row", "rows", "screen", "screens", "section", "sections", "tab", "tabs",
                    ]),
                )),
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

        let mut correction = span.get_content(src).to_vec();

        const AE_IDX: usize = 4;

        *correction.get_mut(AE_IDX)? = match correction.get(AE_IDX)? {
            'a' => 'e',
            'A' => 'E',
            'e' => 'a',
            'E' => 'A',
            _ => return None,
        };

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

    // Prepositions that after "summery" indicate it should likely have been "summary"

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

    // Nouns that usually collocate after "summary"

    #[test]
    fn summery_box() {
        assert_suggestion_result(
            "How to remove summery box in magento review form",
            SummarySummery::default(),
            "How to remove summary box in magento review form",
        );
    }

    #[test]
    fn summery_boxes() {
        assert_suggestion_result(
            "those summery boxes are quite inviting because they do not only show the logo but also an example picture of the speaker/receiver",
            SummarySummery::default(),
            "those summary boxes are quite inviting because they do not only show the logo but also an example picture of the speaker/receiver",
        );
    }

    #[test]
    fn summery_control() {
        assert_suggestion_result(
            "have successfully implemented this for the Validation Summery control, input controls and ValidationToolTips",
            SummarySummery::default(),
            "have successfully implemented this for the Validation Summary control, input controls and ValidationToolTips",
        );
    }

    #[test]
    fn summery_div() {
        assert_suggestion_result(
            "does form and summery DIV in same view ?",
            SummarySummery::default(),
            "does form and summary DIV in same view ?",
        );
    }

    #[test]
    fn summery_element() {
        assert_suggestion_result(
            "How use summery element for label in cell of table view?",
            SummarySummery::default(),
            "How use summary element for label in cell of table view?",
        );
    }

    #[test]
    fn summery_field() {
        assert_suggestion_result(
            "Option without the summery field (if we can automate it via the description).",
            SummarySummery::default(),
            "Option without the summary field (if we can automate it via the description).",
        );
    }

    #[test]
    fn summery_fields() {
        assert_suggestion_result(
            "Add missing summery fields whenever is required",
            SummarySummery::default(),
            "Add missing summary fields whenever is required",
        );
    }

    #[test]
    fn summery_field_all_caps() {
        assert_suggestion_result(
            "The event with empty SUMMERY-field is not imported.",
            SummarySummery::default(),
            "The event with empty SUMMARY-field is not imported.",
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
    fn summery_object() {
        assert_suggestion_result(
            "I could print the filerev summery object between the filerev task and the usemin task",
            SummarySummery::default(),
            "I could print the filerev summary object between the filerev task and the usemin task",
        );
    }

    #[test]
    fn summery_hyphen_page() {
        assert_suggestion_result(
            "The heat-map on the summery-page in the month-overview (stats) ignores the first 10 days.",
            SummarySummery::default(),
            "The heat-map on the summary-page in the month-overview (stats) ignores the first 10 days.",
        );
    }

    #[test]
    fn summery_row() {
        assert_suggestion_result(
            "Automatically adding a month-summery row at the end of each month?",
            SummarySummery::default(),
            "Automatically adding a month-summery row at the end of each month?",
        );
    }

    #[test]
    fn summery_screen() {
        assert_suggestion_result(
            "On the summery screen the four graphs at the bottom (Energy, Thermals, Disk,Network) are constantly resizing",
            SummarySummery::default(),
            "On the summary screen the four graphs at the bottom (Energy, Thermals, Disk,Network) are constantly resizing",
        );
    }

    #[test]
    fn summery_section() {
        assert_suggestion_result(
            "How can I change WooCommerce checkout page order summery section style?",
            SummarySummery::default(),
            "How can I change WooCommerce checkout page order summary section style?",
        );
    }

    #[test]
    fn summery_tab() {
        assert_suggestion_result(
            "double click on that bar and it should give you more infomation in the Summery tab",
            SummarySummery::default(),
            "double click on that bar and it should give you more infomation in the Summary tab",
        );
    }
}
