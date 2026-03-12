use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::ModalVerb,
};

pub struct NorModalPronoun {
    expr: Box<dyn Expr>,
}

impl Default for NorModalPronoun {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::aco("nor")
                    .t_ws()
                    .then_subject_pronoun()
                    .t_ws()
                    .then(ModalVerb::with_common_errors()),
            ),
        }
    }
}

impl ExprLinter for NorModalPronoun {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects the order of the pronoun and modal verb after `nor`."
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
        if ctx
            .map(|(pre, _)| {
                // Check for pattern 1: subject pronoun [ws]
                let is_subj_pronoun = pre
                    .get_rel(-1)
                    .filter(|t| t.kind.is_whitespace())
                    .and_then(|_| pre.get_rel(-2))
                    .is_some_and(|t| t.kind.is_subject_pronoun());

                // Check for pattern 2: possessive [ws] noun [ws]
                let is_poss_and_noun = pre
                    .get_rel(-1)
                    .filter(|t| t.kind.is_whitespace())
                    .and_then(|_| pre.get_rel(-2))
                    .filter(|t| t.kind.is_noun())
                    .and_then(|_| pre.get_rel(-3))
                    .filter(|t| t.kind.is_whitespace())
                    .and_then(|_| pre.get_rel(-4))
                    .is_some_and(|t| t.kind.is_possessive_determiner());

                is_subj_pronoun || is_poss_and_noun
            })
            .unwrap_or(false)
        {
            return None;
        }

        let (pron_tok, modal_tok) = (toks.get_rel(-3)?, toks.get_rel(-1)?);
        let pron_ws_modal_toks = toks.get_rel_slice(-3, -1)?;
        let (pron_span, modal_span) = (pron_tok.span, modal_tok.span);
        let pron_modal_span = pron_ws_modal_toks.span()?;

        let value = format!(
            "{} {}",
            modal_span.get_content_string(src),
            pron_span.get_content_string(src)
        )
        .chars()
        .collect();

        // Avoid capitalizing the modals verbs just because the pronoun was "I"
        let suggestion = if pron_span.get_content(src) == ['I'] {
            Suggestion::ReplaceWith(value)
        } else {
            Suggestion::replace_with_match_case(value, pron_modal_span.get_content(src))
        };

        Some(Lint {
            span: pron_modal_span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![suggestion],
            message: "After `nor`, the modal verb should come before the pronoun.".to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::NorModalPronoun;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn fix_nor_i_can() {
        assert_suggestion_result(
            "i can't see the menu nor i can see the features of your app or how it looks !",
            NorModalPronoun::default(),
            "i can't see the menu nor can i see the features of your app or how it looks !",
        );
    }

    #[test]
    fn fix_nor_i_could() {
        assert_suggestion_result(
            "but never saw any warnings nor I could read messages until I debugged",
            NorModalPronoun::default(),
            "but never saw any warnings nor could I read messages until I debugged",
        );
    }

    #[test]
    fn fix_nor_i_will() {
        assert_suggestion_result(
            "I am not the author of the plugins nor I will be updating bugged/unavailable plugins.",
            NorModalPronoun::default(),
            "I am not the author of the plugins nor will I be updating bugged/unavailable plugins.",
        );
    }

    #[test]
    fn fix_nor_i_would() {
        assert_suggestion_result(
            "I would not like to own a Pollock, nor I would hang one of his paintings on a wall inside my home",
            NorModalPronoun::default(),
            "I would not like to own a Pollock, nor would I hang one of his paintings on a wall inside my home",
        );
    }

    #[test]
    fn fix_nor_it_can() {
        assert_suggestion_result(
            "However, since several days ago FreeTube simply doesn't open ANY videos nor it can search.",
            NorModalPronoun::default(),
            "However, since several days ago FreeTube simply doesn't open ANY videos nor can it search.",
        );
    }

    #[test]
    fn fix_nor_it_should() {
        assert_suggestion_result(
            "Since the code doesn't guard against it (nor it should), internalModule.stripBOM is called with an undefined",
            NorModalPronoun::default(),
            "Since the code doesn't guard against it (nor should it), internalModule.stripBOM is called with an undefined",
        );
    }

    #[test]
    fn fix_nor_it_will() {
        assert_suggestion_result(
            "It will never \"create a table\", nor it will issue any query - it will only create the entity instance.",
            NorModalPronoun::default(),
            "It will never \"create a table\", nor will it issue any query - it will only create the entity instance.",
        );
    }

    #[test]
    fn fix_nor_it_would() {
        assert_suggestion_result(
            "Neither whitespace (excepting NL and CR) is special char in this sense, nor it would destroy something, if it gets \"escaped\" as variable",
            NorModalPronoun::default(),
            "Neither whitespace (excepting NL and CR) is special char in this sense, nor would it destroy something, if it gets \"escaped\" as variable",
        );
    }

    #[test]
    fn fix_nor_they_can() {
        assert_suggestion_result(
            "Currently these assets don't include the code provided by the submodules nor they can be disabled",
            NorModalPronoun::default(),
            "Currently these assets don't include the code provided by the submodules nor can they be disabled",
        );
    }

    #[test]
    fn fix_nor_we_can() {
        assert_suggestion_result(
            "The NSLayoutConstraint errors are really Apple bugs, not our fault, nor we can fix them, but they are harmless.",
            NorModalPronoun::default(),
            "The NSLayoutConstraint errors are really Apple bugs, not our fault, nor can we fix them, but they are harmless.",
        );
    }

    #[test]
    fn fix_nor_you_can() {
        assert_suggestion_result(
            "You cannot create a view to do it, nor you can have a function to do it",
            NorModalPronoun::default(),
            "You cannot create a view to do it, nor can you have a function to do it",
        );
    }

    #[test]
    fn fix_nor_you_should() {
        assert_suggestion_result(
            "I believe you cannot create two sessions through one signin, and maybe nor you should.",
            NorModalPronoun::default(),
            "I believe you cannot create two sessions through one signin, and maybe nor should you.",
        );
    }

    // Potential false positives

    #[test]
    fn ignore_neither_they_nor_i_could() {
        assert_no_lints(
            "One of my users was unable to install tools via mise, but neither they nor I could initially figure out why.",
            NorModalPronoun::default(),
        );
    }

    #[test]
    fn ignore_neither_my_tool_nor_i_shall() {
        assert_no_lints(
            "but neither my tool nor I shall feel disrespected",
            NorModalPronoun::default(),
        );
    }
}
