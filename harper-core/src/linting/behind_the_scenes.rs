use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct BehindTheScenes {
    expr: Box<dyn Expr>,
}

impl Default for BehindTheScenes {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::aco("behind")
                    .t_ws_h()
                    .t_aco("the")
                    .t_ws_h()
                    .t_aco("scene"),
            ),
        }
    }
}

impl ExprLinter for BehindTheScenes {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects `behind the scene` to `behind the scenes`."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.last()?.span;
        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: [Suggestion::replace_with_match_case_str(
                "scenes",
                span.get_content(src),
            )]
            .to_vec(),
            message: "This idiom uses the plural `scenes`.".to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{behind_the_scenes::BehindTheScenes, tests::assert_suggestion_result};

    #[test]
    fn fix_work_bts() {
        assert_suggestion_result(
            "How does this tool work behind the scene.",
            BehindTheScenes::default(),
            "How does this tool work behind the scenes.",
        );
    }
}
