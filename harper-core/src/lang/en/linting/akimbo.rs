use crate::{
    Lint, Token, TokenStringExt,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct Akimbo {
    expr: All,
}

impl Default for Akimbo {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set([
                "arm", "arms", "elbows", "all", "fingers", "hand", "hands", "legs", "limbs",
                "stand", "standing", "stands", "stood",
            ])
            .t_ws_h()
            .t_aco("a")
            .t_ws_h()
            .t_aco("kimbo")
            .but_not(
                SequenceExpr::anything()
                    .t_any()
                    .t_any()
                    .t_any()
                    .t_any()
                    .t_ws()
                    .t_set(["slice"]),
            ),
        }
    }
}

impl ExprLinter for Akimbo {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.get(2..=4)?.span()?;

        let a = toks.get(2)?;
        let kimbo = toks.get(4)?;

        let fix: Vec<char> = a
            .get_ch(src)
            .iter()
            .copied()
            .chain(kimbo.get_ch(src).iter().copied())
            .collect();

        let suggestions = vec![Suggestion::ReplaceWith(fix)];

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "Did you mean `akimbo` (standing with both hands on hips)?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `a kimbo` to `akimbo`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Akimbo;

    #[test]
    fn fix_legs_all_spaces() {
        assert_suggestion_result(
            "I'm also a fan of the “ on the back and legs all a kimbo”.",
            Akimbo::default(),
            "I'm also a fan of the “ on the back and legs all akimbo”.",
        );
    }

    #[test]
    fn fix_legs_hyphens() {
        assert_suggestion_result(
            "I thought I'd have legs-a-kimbo, but not so.",
            Akimbo::default(),
            "I thought I'd have legs-akimbo, but not so.",
        );
    }

    #[test]
    fn fix_limbs() {
        assert_suggestion_result(
            "Just lying there, limbs a kimbo.",
            Akimbo::default(),
            "Just lying there, limbs akimbo.",
        );
    }

    #[test]
    fn fix_letters() {
        assert_suggestion_result(
            "Them syllables are all a-kimbo....",
            Akimbo::default(),
            "Them syllables are all akimbo....",
        );
    }
}
