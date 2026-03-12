use crate::{
    CharStringExt, Lint, Token, TokenKind,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Suggestion, expr_linter::Chunk},
    patterns::ModalVerb,
};

pub struct ModalBeAdjective {
    expr: Box<dyn Expr>,
}

impl Default for ModalBeAdjective {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::with(ModalVerb::default())
                    .t_ws()
                    .then_kind_is_but_isnt_any_of_except(
                        TokenKind::is_adjective,
                        &[
                            TokenKind::is_verb_lemma,  // set
                            TokenKind::is_adverb,      // ever
                            TokenKind::is_preposition, // on
                            TokenKind::is_determiner,  // all
                            TokenKind::is_pronoun,     // all
                        ] as &[_],
                        &[
                            "backup", // adjective commonly misused as a verb
                            "likely", // adjective but with special usage
                        ] as &[_],
                    ),
            ),
        }
    }
}

impl ExprLinter for ModalBeAdjective {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if let Some((_, after)) = ctx
            && after.len() >= 2
            && after[0].kind.is_whitespace()
        {
            // If the 'after' context is whitespace followed by a noun, there is no error
            // (Not including these marginal nouns: "at", "by", "if")
            if after[1].kind.is_noun()
                && !after[1]
                    .span
                    .get_content(src)
                    .eq_any_ignore_ascii_case_str(&["at", "by", "if"])
            {
                return None;
            }

            // If the adjective plus the next word is "kind of"
            if toks
                .last()
                .unwrap()
                .span
                .get_content_string(src)
                .eq_ignore_ascii_case("kind")
                && after[1]
                    .span
                    .get_content(src)
                    .eq_ignore_ascii_case_str("of")
            {
                return None;
            }
        }

        Some(Lint {
            span: toks[0].span,
            suggestions: vec![Suggestion::InsertAfter(" be".chars().collect())],
            message: "You may be missing the word `be` between this modal verb and adjective."
                .to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Looks for `be` missing between a modal verb and adjective."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ModalBeAdjective;

    #[test]
    fn fix_would_nice() {
        assert_suggestion_result(
            "It would nice if Harper could detect this.",
            ModalBeAdjective::default(),
            "It would be nice if Harper could detect this.",
        );
    }

    #[test]
    fn fix_could_configured() {
        assert_suggestion_result(
            "It could configured by parameters and the commands above effectively disable it.",
            ModalBeAdjective::default(),
            "It could be configured by parameters and the commands above effectively disable it.",
        );
    }

    #[test]
    fn fix_will_accessible() {
        assert_suggestion_result(
            "Your WordPress site will accessible at http://localhost",
            ModalBeAdjective::default(),
            "Your WordPress site will be accessible at http://localhost",
        );
    }

    #[test]
    fn ignore_would_external_traffic() {
        assert_no_lints(
            "And why would external traffic be trying to access my server if I don't know who or what it is?",
            ModalBeAdjective::default(),
        )
    }

    #[test]
    fn ignore_could_kind_of() {
        assert_no_lints("you could kind of see the ...", ModalBeAdjective::default())
    }

    // Known false positives. You may want to improve the code to handle some of these.

    #[test]
    #[ignore = "false positive: 'backup' is an adjective but also a spello for the verb 'back up'"]
    fn ignore_you_can_backup() {
        assert_no_lints("You can backup Userdata.", ModalBeAdjective::default());
    }

    #[test]
    #[ignore = "false positive: 'incorrect' should be 'incorrectly'."]
    fn ignore_would_incorrect() {
        assert_no_lints(
            "Bug in versions 4.0 and 4.1 would incorrect list the address module",
            ModalBeAdjective::default(),
        );
    }

    #[test]
    #[ignore = "false positive: 'upper-bound' is an ad-hoc verb here."]
    fn ignore_should_upper() {
        assert_no_lints(
            "we should upper-bound it to the next MAJOR version.",
            ModalBeAdjective::default(),
        );
        assert_no_lints(
            "some older software (filezilla on debian-stable) cannot passive-mode with TLS",
            ModalBeAdjective::default(),
        );
    }
}
