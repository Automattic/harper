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
                            TokenKind::is_adjective,
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

    // Don't flag

    #[test]
    fn ignore_than() {
        assert_no_lints("My arm is not longer than my leg.", NoLonger::default())
    }

    // Flag not longer <verb>

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

    // Flag not longer <adjective>

    #[test]
    fn fix_able() {
        assert_suggestion_result(
            "I am not longer able to set multi-cursors in Zed 0.190.6.",
            NoLonger::default(),
            "I am no longer able to set multi-cursors in Zed 0.190.6.",
        );
    }

    #[test]
    fn fix_affordable() {
        assert_suggestion_result(
            "No not Oakland, it's not longer affordable.",
            NoLonger::default(),
            "No not Oakland, it's no longer affordable.",
        );
    }

    #[test]
    fn fix_bad() {
        assert_suggestion_result(
            "How many times does this have to happen before its not longer bad luck?",
            NoLonger::default(),
            "How many times does this have to happen before its no longer bad luck?",
        );
    }

    #[test]
    fn fix_best() {
        assert_suggestion_result(
            "AWS Java V1 is not longer best practice as specified in this Github page",
            NoLonger::default(),
            "AWS Java V1 is no longer best practice as specified in this Github page",
        );
    }

    #[test]
    fn fix_effective() {
        assert_suggestion_result(
            "when you delete those keys from the dict, it is not longer effective",
            NoLonger::default(),
            "when you delete those keys from the dict, it is no longer effective",
        );
    }

    #[test]
    fn fix_empty() {
        assert_suggestion_result(
            "not only set as username, it sets common name as well and is not longer empty",
            NoLonger::default(),
            "not only set as username, it sets common name as well and is no longer empty",
        );
    }

    #[test]
    fn fix_enough() {
        assert_suggestion_result(
            "the message body is not longer enough",
            NoLonger::default(),
            "the message body is no longer enough",
        );
    }

    #[test]
    fn fix_equal() {
        assert_suggestion_result(
            "once the size of the current batch is not longer equal to batch_size , I used the temporary batch",
            NoLonger::default(),
            "once the size of the current batch is no longer equal to batch_size , I used the temporary batch",
        );
    }

    #[test]
    fn fix_equivalent() {
        assert_suggestion_result(
            "the lambda is not longer equivalent to how std::isspace would behave as a unary predicate",
            NoLonger::default(),
            "the lambda is no longer equivalent to how std::isspace would behave as a unary predicate",
        );
    }

    #[test]
    fn fix_free() {
        assert_suggestion_result(
            "so if i understand it correct, myteslamate is not longer free? ",
            NoLonger::default(),
            "so if i understand it correct, myteslamate is no longer free? ",
        );
    }

    #[test]
    fn fix_good() {
        assert_suggestion_result(
            "Just in case that link is not longer good I'll reproduce the code here.",
            NoLonger::default(),
            "Just in case that link is no longer good I'll reproduce the code here.",
        );
    }

    #[test]
    fn fix_near() {
        assert_suggestion_result(
            "reminder that they are not longer near each other",
            NoLonger::default(),
            "reminder that they are no longer near each other",
        );
    }

    #[test]
    fn fix_open() {
        assert_suggestion_result(
            "removing old breakpoints from a project which was not longer open",
            NoLonger::default(),
            "removing old breakpoints from a project which was no longer open",
        );
    }

    #[test]
    fn fix_possible() {
        assert_suggestion_result(
            "As far as I can set tell it is not longer possible to set these programmatically.",
            NoLonger::default(),
            "As far as I can set tell it is no longer possible to set these programmatically.",
        );
    }

    #[test]
    fn fix_relevant() {
        assert_suggestion_result(
            "individual remuneration is not longer relevant as we can produce enough",
            NoLonger::default(),
            "individual remuneration is no longer relevant as we can produce enough",
        );
    }

    #[test]
    fn fix_sufficient() {
        assert_suggestion_result(
            "the fichier.close() command is not longer sufficient to close the file",
            NoLonger::default(),
            "the fichier.close() command is no longer sufficient to close the file",
        );
    }
}
