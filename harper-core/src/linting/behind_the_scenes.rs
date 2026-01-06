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

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if let Some((before, _)) = ctx
            && before.last().is_some_and(|t| t.kind.is_hyphen())
        {
            return None;
        }

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
    use crate::linting::{
        behind_the_scenes::BehindTheScenes,
        tests::{assert_no_lints, assert_suggestion_result},
    };

    #[test]
    fn pluralize_work_bts() {
        assert_suggestion_result(
            "How does this tool work behind the scene.",
            BehindTheScenes::default(),
            "How does this tool work behind the scenes.",
        );
    }

    #[test]
    #[ignore = "Correcting hyphenation is not yet implemented."]
    fn pluralize_and_hyphenate() {
        assert_suggestion_result(
            "So, to open the 'real' behind the scene menu i need to do these steps:",
            BehindTheScenes::default(),
            "So, to open the 'real' behind-the-scenes menu i need to do these steps:",
        );
    }

    #[test]
    fn dont_flag_when_hyphenated_to_previous_word() {
        assert_no_lints(
            "Contribute to techking11/react-behind-the-scene development by creating an account on GitHub.",
            BehindTheScenes::default(),
        );
    }

    #[test]
    fn pluralize_bts_processing() {
        assert_suggestion_result(
            "Behind-the-scene processing details are printed in the Log window.",
            BehindTheScenes::default(),
            "Behind-the-scenes processing details are printed in the Log window.",
        );
    }
}
