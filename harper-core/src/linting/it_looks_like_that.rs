use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};
use crate::token_string_ext::TokenStringExt;

pub struct ItLooksLikeThat {
    expr: Box<dyn Expr>,
}

impl Default for ItLooksLikeThat {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::fixed_phrase("it looks like that")
                    .then_whitespace()
                    .then_kind_where(|kind| {
                        // Heuristics on the word after "that" which show "that" was used
                        // as a relative pronoun, which is a mistake
                        let is_subj = kind.is_subject_pronoun();
                        let is_ing = kind.is_verb_progressive_form();
                        let is_definitely_rel_pron = is_subj || is_ing;

                        // Heuristics on the word after "that" which show "that"
                        // could possibly be a legitimate demonstrative pronoun or determiner
                        // as a demonstrative pronoun or a determiner
                        // which would not be a mistake.
                        let is_v3psgpres = kind.is_verb_third_person_singular_present_form();
                        // NOTE: we don't have .is_modal_verb() but maybe we need it now!
                        let is_vmodal_or_aux = kind.is_auxiliary_verb();
                        let is_vpret = kind.is_verb_simple_past_form();
                        let is_noun = kind.is_noun();
                        let is_oov = kind.is_oov();

                        let maybe_demonstrative_or_determiner =
                            is_v3psgpres || is_vmodal_or_aux || is_vpret || is_noun || is_oov;

                        is_definitely_rel_pron || !maybe_demonstrative_or_determiner
                    }),
            ),
        }
    }
}

impl ExprLinter for ItLooksLikeThat {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], _: &[char]) -> Option<Lint> {
        let that_span = toks[6..8].span()?;

        Some(Lint {
            span: that_span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::Remove],
            message: "`that` is redundant and ungrammatical here".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Corrects `it looks like that` to just `it looks like`."
    }
}

#[cfg(test)]
mod tests {
    mod that_noun {
        use super::super::ItLooksLikeThat;
        use crate::linting::tests::assert_no_lints;

        #[test]
        fn dont_flag_that_noun_is_also_verb_part_of_np() {
            // "that" could be legit demonstrative, indicating which 'file tree view'
            // "file" is both a noun and a verb
            assert_no_lints(
                "It looks like that file tree view is just for things that have already been committed?",
                ItLooksLikeThat::default(),
            );
        }
        #[test]
        fn dont_flag_that_noun_is_also_adj() {
            // "metric" is both a noun and an adjective
            assert_no_lints(
                "Yes, unfortunately it looks like that metric kind isn't supported yet.",
                ItLooksLikeThat::default(),
            );
        }
        #[test]
        fn cant_flag_that_noun_is_also_verb_function() {
            // "that" is not demonstrative, but heuristics can't determine that.
            // "function" is both a noun and a verb
            assert_no_lints(
                "It looks like that function Config.validate_doc_path is only called in one place",
                ItLooksLikeThat::default(),
            );
        }
        #[test]
        fn dont_flag_that_noun_is_also_verb_test() {
            assert_no_lints(
                "It looks like that test runs with -sOFFSCREEN_FRAMEBUFFER",
                ItLooksLikeThat::default(),
            );
        }

        #[test]
        fn dont_flag_that_oov() {
            // "that" could be legit demonstrative, indicating which 'nms'
            // because OOV words are most commonly nouns.
            assert_no_lints(
                "It looks like that nms is not working.",
                ItLooksLikeThat::default(),
            );
        }

        #[test]
        fn dont_flag_that_noun_pad() {
            assert_no_lints(
                "It looks like that pad was not covered in solder mask or glue",
                ItLooksLikeThat::default(),
            );
        }

        #[test]
        fn dont_flag_that_noun_plural() {
            assert_no_lints(
                "The issue we're running into is that it looks like that nodes not only want to peer via raft",
                ItLooksLikeThat::default(),
            );
        }
    }

    mod that_det {
        use super::super::ItLooksLikeThat;
        use crate::linting::tests::assert_suggestion_result;

        #[test]
        fn fix_that_the() {
            // "that" is being wrongly used as a relative pronoun
            assert_suggestion_result(
                "it looks like that the original products should have NULL in the value column",
                ItLooksLikeThat::default(),
                "it looks like the original products should have NULL in the value column",
            );
        }

        #[test]
        fn fix_that_some() {
            assert_suggestion_result(
                "From first expresion it looks like that some tokkens or what was cached",
                ItLooksLikeThat::default(),
                "From first expresion it looks like some tokkens or what was cached",
            );
        }
    }

    mod that_verb {
        use super::super::ItLooksLikeThat;
        use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

        #[test]
        fn dont_flag_that_verb_3p_sing_pres_is() {
            // "that" is definitely legit demonstrative pronoun
            // Because "is" is a linking verb in 3rd person singular present form.
            assert_no_lints(
                "Looking at the code it looks like that is not the case",
                ItLooksLikeThat::default(),
            );
        }

        #[test]
        fn dont_flag_that_verb_3p_sing_pres_comes() {
            assert_no_lints(
                "it looks like that comes with additional compile-time dependencies",
                ItLooksLikeThat::default(),
            );
        }

        #[test]
        fn fix_that_it_verb_lemma() {
            // "that" is being wrongly used as a relative pronoun
            // But it's hard to check becuase 'renovate' is a verb but is being used as a noun
            assert_suggestion_result(
                "It looks like that Renovate decides to not reuse the branch when there are no changes in it",
                ItLooksLikeThat::default(),
                "It looks like Renovate decides to not reuse the branch when there are no changes in it",
            );
        }

        #[test]
        fn dont_flag_that_modal_verb_might() {
            assert_no_lints(
                "It looks like that might be exactly what I needed!",
                ItLooksLikeThat::default(),
            );
        }

        #[test]
        fn dont_flag_that_verb_modal_would() {
            assert_no_lints(
                "but it looks like that would require writing the data out to vsimem and reading it back",
                ItLooksLikeThat::default(),
            );
        }

        #[test]
        fn fix_that_verb_ing_have() {
            // Verbs in -ing are also gerunds, which are nouns.
            // But at least in this case, "having", it doesn't work after "that".
            assert_suggestion_result(
                "It looks like that having <br> tags inside them breaks the rendering",
                ItLooksLikeThat::default(),
                "It looks like having <br> tags inside them breaks the rendering",
            );
        }

        #[test]
        fn fix_that_verb_ing_using() {
            assert_suggestion_result(
                "it looks like that using TensorFlow in conjunction with packages that use pybind11_abseil will fail",
                ItLooksLikeThat::default(),
                "it looks like using TensorFlow in conjunction with packages that use pybind11_abseil will fail",
            );
        }

        #[test]
        fn dont_flag_that_verb_simple_past() {
            assert_no_lints(
                "but it looks like that got accidentally reverted at some point",
                ItLooksLikeThat::default(),
            );
        }
    }

    mod pronoun {
        use super::super::ItLooksLikeThat;
        use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

        #[test]
        fn fix_that_subj_obj_pronoun_it_was() {
            // "that" is being wrongly used as a relative pronoun
            assert_suggestion_result(
                "It looks like that it was not improved a lot.",
                ItLooksLikeThat::default(),
                "It looks like it was not improved a lot.",
            );
        }
        #[test]
        fn fix_that_subj_obj_pronoun_it_works() {
            // "that" is being wrongly used as a relative pronoun
            assert_suggestion_result(
                "Thx, it looks like that it works for Inpainting itself",
                ItLooksLikeThat::default(),
                "Thx, it looks like it works for Inpainting itself",
            );
        }

        #[test]
        fn fix_that_subj_obj_pronoun_you() {
            assert_suggestion_result(
                "It looks like that you can't use the files in combination.",
                ItLooksLikeThat::default(),
                "It looks like you can't use the files in combination.",
            );
        }

        #[test]
        fn dont_flag_thats() {
            assert_no_lints(
                "it looks like that's how you access the system changeset functionality",
                ItLooksLikeThat::default(),
            );
        }
    }

    mod conjunction {
        use super::super::ItLooksLikeThat;
        use crate::linting::tests::assert_no_lints;

        #[test]
        fn cant_flag_that_if() {
            // This can be read two ways, so we can't flag it
            assert_no_lints(
                "It looks like that if the server goes away in the middle of a request, and a request is cancelled",
                ItLooksLikeThat::default(),
            );
        }

        #[test]
        fn cant_flag_that_but() {
            // This can be read two ways, so we can't flag it
            assert_no_lints(
                "Yes, it looks like that but it is unreasonable since the shim executable is in the same directory",
                ItLooksLikeThat::default(),
            );
        }
    }
}
