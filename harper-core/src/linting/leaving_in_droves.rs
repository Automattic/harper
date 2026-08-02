use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct LeavingInDroves {
    expr: SequenceExpr,
}

impl Default for LeavingInDroves {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["leave", "leaves", "leaving", "left"])
                .t_ws()
                .t_aco("in")
                .t_ws()
                .t_aco("drones"),
        }
    }
}

impl ExprLinter for LeavingInDroves {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let span = toks.last()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Eggcorn,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "droves",
                span.get_content(_src),
            )],
            message: "`Drones` is an eggcorn. The correct word is `droves`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `leaving in drones` to `leaving in droves`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::LeavingInDroves;

    #[test]
    fn jobs() {
        assert_suggestion_result(
            "Jobs are leaving in drones out of the country.",
            LeavingInDroves::default(),
            "Jobs are leaving in droves out of the country.",
        );
    }

    #[test]
    fn wealthy() {
        assert_suggestion_result(
            "Yet they are amongst some of the loudest promoters of the notion that the wealthy will leave in drones.",
            LeavingInDroves::default(),
            "Yet they are amongst some of the loudest promoters of the notion that the wealthy will leave in droves.",
        );
    }

    #[test]
    fn employees() {
        assert_suggestion_result(
            "consider the damage that will be caused when employees start leaving in drones as if we didn't have enough people leave already in the past 2 years.",
            LeavingInDroves::default(),
            "consider the damage that will be caused when employees start leaving in droves as if we didn't have enough people leave already in the past 2 years.",
        );
    }

    #[test]
    fn pastors() {
        assert_suggestion_result(
            "Why Are Nigerian Pastors Leaving In Drones?",
            LeavingInDroves::default(),
            "Why Are Nigerian Pastors Leaving In Droves?",
        );
    }

    #[test]
    fn left() {
        assert_suggestion_result(
            "I have remained at the place while my coworkers have left in drones.",
            LeavingInDroves::default(),
            "I have remained at the place while my coworkers have left in droves.",
        );
    }
}
