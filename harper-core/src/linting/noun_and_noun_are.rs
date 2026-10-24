use crate::{
    CharStringExt, Lint, Token, TokenKind, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct NounAndNounAre {
    expr: SequenceExpr,
}

impl Default for NounAndNounAre {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_noun()
                .t_ws()
                .t_aco("and")
                .t_ws()
                // .then_kind_is_but_is_not_except(
                //     TokenKind::is_noun,
                //     TokenKind::is_verb,
                //     &["everyone", "he", "it", "what", "why"],
                // )
                .then(|t: &Token, s: &[char]| {
                    t.kind.is_noun()
                        && !t.kind.is_verb()
                        && !t.kind.is_adjective()
                        && !t
                            .get_ch(s)
                            .eq_any_ignore_ascii_case_str(&["everyone", "he", "it", "what", "why"])
                })
                .t_ws()
                .t_aco("is"),
        }
    }
}

impl ExprLinter for NounAndNounAre {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));
        let span = matched_tokens.span()?;
        let lint_kind = LintKind::Miscellaneous;
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "correction",
            span.get_content(source),
        )];
        let message = "Fix this erorr".to_owned();
        Some(Lint {
            span,
            lint_kind,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter skeleton for contributors to copy into `harper_core/src/linting/` and rename."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::NounAndNounAre;

    #[test]
    fn test_skeleton() {
        assert_suggestion_result("erorr", NounAndNounAre::default(), "correction");
    }
}
