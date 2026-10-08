use crate::{
    Lint, Token, TokenStringExt,
    expr::{All, Expr, FirstMatchOf, SequenceExpr, UnlessStep},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct Themself {
    expr: FirstMatchOf,
}

impl Default for Themself {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(SequenceExpr::aco("their").t_ws().t_aco("selves")) as Box<dyn Expr>,
                Box::new(All::new([
                    Box::new(SequenceExpr::aco("their").t_ws().t_aco("self")) as Box<dyn Expr>,
                    Box::new(UnlessStep::new(
                        SequenceExpr::anything().t_any().t_any().then_any_of([
                            Box::new(SequenceExpr::default().then_hyphen()),
                            Box::new(SequenceExpr::whitespace().then(|t: &Token, _: &[char]| {
                                t.kind.is_noun()
                                    || t.kind.is_verb_past_form()
                                    || t.kind.is_verb_past_participle_form()
                            })),
                        ]),
                        |_: &Token, _: &[char]| true,
                    )),
                ])),
            ]),
        }
    }
}

impl ExprLinter for Themself {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let refl = toks.get(2)?.get_ch(src);

        let span = toks.span()?;

        let suggestions = vec![Suggestion::replace_with_match_case_str(
            if refl.len() == 4 {
                "themself"
            } else {
                "themselves"
            },
            span.get_content(src),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "This should be one word, not two.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `their self` and `their selves` to `themself` and `themselves`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::Themself;

    #[test]
    fn fix_respect_their_self() {
        assert_suggestion_result(
            "You can't respect someone who doesn't respect their self.",
            Themself::default(),
            "You can't respect someone who doesn't respect themself.",
        );
    }

    #[test]
    fn dont_flag_self_hyphen() {
        assert_no_lints(
            "For most renaissance artists, their self-portraits are far from photorealistic.",
            Themself::default(),
        );
    }

    #[test]
    fn fix_their_selves() {
        assert_suggestion_result(
            "Constructedness: see \"people as forming and reforming their selves within each relationship\".",
            Themself::default(),
            "Constructedness: see \"people as forming and reforming themselves within each relationship\".",
        );
    }

    #[test]
    fn dont_flag_self_hosted() {
        assert_no_lints(
            "ZeroTier is certainly another viable alternative, but their self hosted option still relies on the ZeroTier root servers",
            Themself::default(),
        );
    }

    #[test]
    fn dont_flag_self_interest() {
        assert_no_lints(
            "this state that everyone is acting in their self interest is not something we should promote",
            Themself::default(),
        );
    }

    #[test]
    fn dont_flag_self_proclaimed() {
        assert_no_lints(
            "Which is why they feel threatened by anything that challenges their self proclaimed uniquely human trait.",
            Themself::default(),
        );
    }
}
