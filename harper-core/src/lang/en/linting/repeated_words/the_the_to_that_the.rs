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
            expr: SequenceExpr::word_set([
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
            message: "Did you mean `that the` or just `the`?".to_owned(),
            // Higher priority (lower number) than `RepeatedWordsGeneral`
            priority: 126,
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `the the` to `that the` or to a single `the`."
    }
}
