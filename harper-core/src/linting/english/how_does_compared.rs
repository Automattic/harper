use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct HowDoesCompared {
    expr: SequenceExpr,
}

impl Default for HowDoesCompared {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("how")
                .t_ws()
                .then_any_of([
                    Box::new(
                        SequenceExpr::aco("do")
                            .t_ws()
                            .t_set(&["these", "they", "those"]),
                    ) as Box<dyn Expr>,
                    Box::new(
                        SequenceExpr::aco("does")
                            .t_ws()
                            .t_set(&["it", "that", "this"]),
                    ),
                    Box::new(
                        SequenceExpr::aco("did")
                            .t_ws()
                            .t_set(&["it", "that", "these", "they", "this", "those"]),
                    ),
                ])
                .t_ws()
                .t_set(&["compared", "compares"]),
        }
    }
}

impl ExprLinter for HowDoesCompared {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.last()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "compare",
                span.get_content(src),
            )],
            message: "Use the base form of the verb `compare`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `how do/does/did X compared/compares to Y` to use `compare`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::HowDoesCompared;

    #[test]
    fn fix_does_it_compared() {
        assert_suggestion_result(
            "Oo how does it compared to matrix chatGPT bot",
            HowDoesCompared::default(),
            "Oo how does it compare to matrix chatGPT bot",
        );
    }

    #[test]
    fn fix_does_it_compares() {
        assert_suggestion_result(
            "How does it compares to openspec or other spec driven frameworks?",
            HowDoesCompared::default(),
            "How does it compare to openspec or other spec driven frameworks?",
        );
    }

    #[test]
    fn fix_does_that_compared() {
        assert_suggestion_result(
            "Also I wonder if your touchscreen works with VoodooRMI, or how does that compared to VoodooI2CHID?",
            HowDoesCompared::default(),
            "Also I wonder if your touchscreen works with VoodooRMI, or how does that compare to VoodooI2CHID?",
        );
    }

    #[test]
    fn fix_does_that_compares() {
        assert_suggestion_result(
            "lets take a look at where we are as per the latest SBP payment system report and how does that compares with the historical trend for these figures",
            HowDoesCompared::default(),
            "lets take a look at where we are as per the latest SBP payment system report and how does that compare with the historical trend for these figures",
        );
    }

    #[test]
    fn fix_does_this_compared() {
        assert_suggestion_result(
            "How does this compared to other softwares like 3d-agent , threedle or SAM 3d from meta?",
            HowDoesCompared::default(),
            "How does this compare to other softwares like 3d-agent , threedle or SAM 3d from meta?",
        );
    }

    #[test]
    fn fix_does_this_compares() {
        assert_suggestion_result(
            "How does this compares to the RetinaFace?",
            HowDoesCompared::default(),
            "How does this compare to the RetinaFace?",
        );
    }
}
