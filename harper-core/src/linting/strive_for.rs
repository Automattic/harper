use crate::{
    Lint, Token, TokenKind,
    expr::{AnchorEnd, Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct StriveFor {
    expr: SequenceExpr,
}

impl Default for StriveFor {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["strive", "strived", "striven", "strives", "striving"])
                .t_ws()
                .t_aco("to")
                .then_any_of([
                    Box::new(AnchorEnd) as Box<dyn Expr>,
                    Box::new(SequenceExpr::whitespace().then_kind_any_but_not(
                        &[
                            TokenKind::is_noun,
                            TokenKind::is_adjective,
                            TokenKind::is_determiner,
                        ],
                        TokenKind::is_verb,
                    )),
                ]),
        }
    }
}

impl ExprLinter for StriveFor {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.get(2)?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "for",
                span.get_content(source),
            )],
            message: "Use `for` when `strive` is followed by a noun, or add a verb after `to`."
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `strive to` to `strive for` when not followed by a verb."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::StriveFor;

    // Contrived basic tests

    #[test]
    fn fix_at_end() {
        assert_suggestion_result(
            "What we should strive to.",
            StriveFor::default(),
            "What we should strive for.",
        );
    }

    // Real-world examples

    #[test]
    fn fix_before_article() {
        assert_suggestion_result(
            "No solution will be perfect, but we should strive to a solution that's better than doing nothing.",
            StriveFor::default(),
            "No solution will be perfect, but we should strive for a solution that's better than doing nothing.",
        );
    }

    #[test]
    fn fix_before_adjective() {
        assert_suggestion_result(
            "We strive to a high level of sustainability performance at our operated assets",
            StriveFor::default(),
            "We strive for a high level of sustainability performance at our operated assets",
        );
    }

    #[test]
    fn fix_before_comparative() {
        assert_suggestion_result(
            "will certainly motivate them towards striving to greater heights",
            StriveFor::default(),
            "will certainly motivate them towards striving for greater heights",
        );
    }

    #[test]
    fn fix_before_noun() {
        assert_suggestion_result(
            "Nearly 2000 fire fighters came to Strive to Excellence to get better.",
            StriveFor::default(),
            "Nearly 2000 fire fighters came to Strive for Excellence to get better.",
        );
    }

    #[test]
    fn dont_flag_before_word_thats_both_noun_and_verb() {
        assert_no_lints(
            "We love much better those who endeavor to imitate us, than those who strive to equal us.",
            StriveFor::default(),
        );
    }
}
