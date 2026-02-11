use crate::{
    Lint, Token, TokenKind,
    expr::{Expr, OwnedExprExt, SequenceExpr},
    linting::{Chunk, ExprLinter, LintKind, Suggestion},
};

pub struct NoLonger {
    expr: Box<dyn Expr>,
}

impl Default for NoLonger {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::aco("not")
                    .t_ws()
                    .t_aco("longer")
                    .then_optional(SequenceExpr::default().t_ws().then_kind_any(
                        &[
                            TokenKind::is_verb_lemma,
                            TokenKind::is_verb_third_person_singular_present_form,
                            TokenKind::is_verb_past_participle_form,
                            TokenKind::is_verb_progressive_form,
                        ][..],
                    ))
                    .and_not(
                        SequenceExpr::anything()
                            .t_any()
                            .t_any()
                            .t_any()
                            .t_aco("than"),
                    ),
            ),
        }
    }
}

impl ExprLinter for NoLonger {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn description(&self) -> &str {
        "Corrects `not longer` when it should be `no longer`."
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks[0].span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "no",
                toks[0].span.get_content(src),
            )],
            message: "The correct expression is `no longer`.".to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::NoLonger;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn ignore_than() {
        assert_no_lints("My arm is not longer than my leg.", NoLonger::default())
    }

    #[test]
    fn fix_can_modal() {
        // TODO: would an improvement always be? - no longer can -> can no longer
        assert_suggestion_result(
            "and I've found out that I not longer can launch Kitty from the menus",
            NoLonger::default(),
            "and I've found out that I no longer can launch Kitty from the menus",
        );
    }

    #[test]
    fn fix_done_past_participle() {
        assert_suggestion_result(
            "I've noticed that the ML stuff is not longer done on the more recent photos.",
            NoLonger::default(),
            "I've noticed that the ML stuff is no longer done on the more recent photos.",
        );
    }

    #[test]
    fn fix_exist() {
        assert_suggestion_result(
            "Vendoring means that the transitive dependencies do not longer exist from the point of view of the consumer.",
            NoLonger::default(),
            "Vendoring means that the transitive dependencies do no longer exist from the point of view of the consumer.",
        );
    }

    #[test]
    fn fix_exists_3rd_person_singular_present() {
        assert_suggestion_result(
            "this script is mentioned in the RF3 Readme but the script not longer exists",
            NoLonger::default(),
            "this script is mentioned in the RF3 Readme but the script no longer exists",
        );
    }

    #[test]
    fn fix_render() {
        assert_suggestion_result(
            "auto comments will not longer render annotations in such a way as to make them valid annotation links",
            NoLonger::default(),
            "auto comments will no longer render annotations in such a way as to make them valid annotation links",
        );
    }

    #[test]
    fn fix_saved_regular_past() {
        assert_suggestion_result(
            "edit notes are not longer saved on mobile",
            NoLonger::default(),
            "edit notes are no longer saved on mobile",
        );
    }

    #[test]
    fn fix_saving_present_participle() {
        assert_suggestion_result(
            "After Updating to 4.3.2 from 4.2.1 the JSON Editor is not longer saving the metadata.",
            NoLonger::default(),
            "After Updating to 4.3.2 from 4.2.1 the JSON Editor is no longer saving the metadata.",
        );
    }

    #[test]
    fn fix_written_past_participle() {
        assert_suggestion_result(
            "I get this error and the fasta file is not longer written",
            NoLonger::default(),
            "I get this error and the fasta file is no longer written",
        );
    }
}
