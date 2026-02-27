use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct AllHellBreakLoose {
    expr: Box<dyn Expr>,
}

impl Default for AllHellBreakLoose {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::aco("all")
                    .t_ws()
                    .t_aco("hell")
                    .t_ws()
                    .then_word_set(&["break", "breaking", "breaks", "broke", "broken"])
                    .t_ws()
                    .t_aco("out"),
            ),
        }
    }
}

impl ExprLinter for AllHellBreakLoose {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let outtok = toks.last()?;
        let outspan = outtok.span;
        let outchars = outspan.get_content(src);

        Some(Lint {
            lint_kind: LintKind::Eggcorn,
            span: outspan,
            suggestions: vec![Suggestion::replace_with_match_case_str("loose", outchars)],
            message: "The correct idiom is `all hell breaks loose`.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects forms of `all hell breaks out` to `all hell breaks loose`."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::AllHellBreakLoose;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn fix_break() {
        assert_suggestion_result(
            "Just run around planting satchels charges while you're alive and let all hell break out when you die.",
            AllHellBreakLoose::default(),
            "Just run around planting satchels charges while you're alive and let all hell break loose when you die.",
        );
    }

    #[test]
    fn fix_breaks() {
        assert_suggestion_result(
            "we upgraded 2 months ago, and now we upgrade prod and all hell breaks out",
            AllHellBreakLoose::default(),
            "we upgraded 2 months ago, and now we upgrade prod and all hell breaks loose",
        );
    }

    #[test]
    fn fix_breaking() {
        assert_suggestion_result(
            "Next scene, all hell breaking out!",
            AllHellBreakLoose::default(),
            "Next scene, all hell breaking loose!",
        );
    }

    #[test]
    fn fix_broke() {
        assert_suggestion_result(
            "this time going from 1.3.4 to 1.4.2 and all hell broke out",
            AllHellBreakLoose::default(),
            "this time going from 1.3.4 to 1.4.2 and all hell broke loose",
        );
    }

    #[test]
    fn fix_broken() {
        assert_suggestion_result(
            "I’m using silenced weapons but as soon as I fire it’s all hell broken out.",
            AllHellBreakLoose::default(),
            "I’m using silenced weapons but as soon as I fire it’s all hell broken loose.",
        );
    }
}
