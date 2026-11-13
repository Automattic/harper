use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct GoTheThe {
    expr: SequenceExpr,
}

impl Default for GoTheThe {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["go", "goes", "going", "gone", "went"])
                .t_ws()
                .then_word_seq(&["the", "the"]),
        }
    }
}

impl ExprLinter for GoTheThe {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.get(2)?.span;
        Some(Lint {
            span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "to",
                span.get_content(src),
            )],
            message: "Did you mean `go to the`?".to_owned(),
            // Higher priority (lower number) than `RepeatedWordsGeneral`
            priority: 126,
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `go the the` to `go to the`."
    }
}
