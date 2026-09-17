use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct GetPassGoPass {
    expr: SequenceExpr,
}

impl Default for GetPassGoPass {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set([
                "get", "gets", "getting", "got", "gotten", "go", "goes", "going", "gone", "went",
            ])
            .t_ws()
            .t_aco("pass"),
        }
    }
}

impl ExprLinter for GetPassGoPass {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.last()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "past",
                span.get_content(src),
            )],
            message: "Use the preposition `past` here and not the verb `pass`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `pass` to `past` after `get` and `go`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::GetPassGoPass;

    #[test]
    fn fix_get_pass() {
        assert_suggestion_result(
            "Can't get pass the provider confirmation in the exchange feature",
            GetPassGoPass::default(),
            "Can't get past the provider confirmation in the exchange feature",
        );
    }

    #[test]
    fn fix_gets_pass() {
        assert_suggestion_result(
            "But once rewritten in block code it gets pass the type check just fine",
            GetPassGoPass::default(),
            "But once rewritten in block code it gets past the type check just fine",
        );
    }

    #[test]
    fn fix_getting_pass() {
        assert_suggestion_result(
            "I am not getting pass the initial \"Control the game using WASD keys\" message.",
            GetPassGoPass::default(),
            "I am not getting past the initial \"Control the game using WASD keys\" message.",
        );
    }

    #[test]
    fn fix_go_pass() {
        assert_suggestion_result(
            "the X axis movement will go pass the collider",
            GetPassGoPass::default(),
            "the X axis movement will go past the collider",
        );
    }

    #[test]
    fn fix_goes_pass() {
        assert_suggestion_result(
            "return false if it goes pass the number of questions",
            GetPassGoPass::default(),
            "return false if it goes past the number of questions",
        );
    }

    #[test]
    fn fix_going_pass() {
        assert_suggestion_result(
            "Why is it not going pass the first batch?",
            GetPassGoPass::default(),
            "Why is it not going past the first batch?",
        )
    }

    #[test]
    fn fix_gone_pass() {
        assert_suggestion_result(
            "Make sure we haven't gone pass the node count limit.",
            GetPassGoPass::default(),
            "Make sure we haven't gone past the node count limit.",
        )
    }

    #[test]
    fn fix_got_pass() {
        assert_suggestion_result(
            "Ok I got pass the first issue.",
            GetPassGoPass::default(),
            "Ok I got past the first issue.",
        );
    }

    #[test]
    fn fix_gotten_pass() {
        assert_suggestion_result(
            "I have now gotten pass the show stoppers.",
            GetPassGoPass::default(),
            "I have now gotten past the show stoppers.",
        )
    }
}
