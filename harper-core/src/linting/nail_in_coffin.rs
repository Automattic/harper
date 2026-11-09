use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

pub struct NailInCoffin {
    expr: SequenceExpr,
}

impl Default for NailInCoffin {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["final", "last"])
                .t_ws()
                .t_set(["nail", "nails"])
                .t_ws()
                .t_aco("on")
                .t_ws()
                .then_determiner()
                .t_ws()
                .t_set(["coffin", "coffins"]),
        }
    }
}

impl ExprLinter for NailInCoffin {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span =
            find_the_only_token_matching(toks, src, |t, c| t.get_ch(c).eq_ch(&['o', 'n']))?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "in",
                span.get_content(src),
            )],
            message: "This idiom uses the preposition `in` rather than `on`".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `nail on the coffin` to `nail in the coffin`"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::NailInCoffin;

    #[test]
    fn fix_final_the() {
        assert_suggestion_result(
            "Maybe this will be the final nail on the coffin for the console.",
            NailInCoffin::default(),
            "Maybe this will be the final nail in the coffin for the console.",
        );
    }

    #[test]
    fn fix_final_his() {
        assert_suggestion_result(
            "A single hit whether blocked or not is going to put the final nail on his coffin.",
            NailInCoffin::default(),
            "A single hit whether blocked or not is going to put the final nail in his coffin.",
        );
    }

    #[test]
    fn fix_last_his() {
        assert_suggestion_result(
            "and as the last nail on his coffin",
            NailInCoffin::default(),
            "and as the last nail in his coffin",
        );
    }

    #[test]
    fn fix_final_her() {
        assert_suggestion_result(
            "That's literally the final nail on her coffin since she's the dead ass only one that can't use that card.",
            NailInCoffin::default(),
            "That's literally the final nail in her coffin since she's the dead ass only one that can't use that card.",
        );
    }

    #[test]
    fn fix_last_their() {
        assert_suggestion_result(
            "So I just sent Khalida to put the last nail on their coffin",
            NailInCoffin::default(),
            "So I just sent Khalida to put the last nail in their coffin",
        );
    }
}
