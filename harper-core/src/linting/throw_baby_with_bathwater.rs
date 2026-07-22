/// See also the linter `ThrowRubbish` for a related pattern.
use crate::{
    CharStringExt, Lint, Lrc, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::Word,
};

pub struct ThrowBabyWithBathwater {
    expr: SequenceExpr,
}

impl Default for ThrowBabyWithBathwater {
    fn default() -> Self {
        let the_sp_baby_sp = Lrc::new(SequenceExpr::word_seq(&["the", "baby"]).t_ws());
        let away_sp = Lrc::new(SequenceExpr::word_seq(&["away"]).t_ws());

        Self {
            expr: SequenceExpr::word_set(&[
                // Correct spellings
                "throw", "threw", "thrown", "throwing", "throws",
                // Tolerate common misspelling, leaving it for its own linter to correct
                "through",
            ])
            .t_ws()
            .then_any_of([
                Box::new(the_sp_baby_sp.clone()) as Box<dyn Expr>,
                Box::new(SequenceExpr::with(away_sp.clone()).then(the_sp_baby_sp.clone())),
                Box::new(SequenceExpr::with(the_sp_baby_sp.clone()).then(away_sp)),
            ])
            .then_word_seq(&["with", "the"])
            .t_ws()
            .then_any_of([
                Box::new(Word::new("bathwater")) as Box<dyn Expr>,
                Box::new(SequenceExpr::word_seq(&["bath", "water"])),
            ]),
        }
    }
}

impl ExprLinter for ThrowBabyWithBathwater {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let away_idx: Option<usize> = [2, 6]
            .into_iter()
            .find(|&i| toks[i].get_ch(src).eq_str("away"));

        match away_idx {
            Some(idx) => {
                let away_tok = &toks[idx];

                Some(Lint {
                    span: away_tok.span,
                    lint_kind: LintKind::Nonstandard,
                    suggestions: vec![Suggestion::replace_with_match_case_str(
                        "out",
                        away_tok.get_ch(src),
                    )],
                    message: "This idiom uses `throw out` rather than `throw away`.".to_owned(),
                    ..Default::default()
                })
            }
            None => {
                let the_sp_baby_toks = &toks[2..=4];
                let the_sp_baby_span = the_sp_baby_toks.span()?;
                let the_sp_baby_chars = the_sp_baby_toks.get_ch(src)?;

                let the_baby_out: Vec<char> = ['o', 'u', 't', ' ']
                    .iter()
                    .copied()
                    .chain(the_sp_baby_chars.iter().copied())
                    .collect();

                let out_the_baby: Vec<char> = the_sp_baby_chars
                    .iter()
                    .copied()
                    .chain([' ', 'o', 'u', 't'].iter().copied())
                    .collect();

                let span = the_sp_baby_span;
                let template = span.get_content(src);
                let suggestions = vec![
                    Suggestion::replace_with_match_case(the_baby_out, template),
                    Suggestion::replace_with_match_case(out_the_baby, template),
                ];

                Some(Lint {
                    span,
                    lint_kind: LintKind::Usage,
                    suggestions,
                    message: "This idiom requires `throw out` rathern than just `throw`."
                        .to_owned(),
                    ..Default::default()
                })
            }
        }
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects wrong or nonstandard variants of the idiom 'to throw the baby out with the bathwater'"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
    };

    use super::ThrowBabyWithBathwater;

    #[test]
    fn fix_almost_throwing_the_baby_bathwater() {
        assert_good_and_bad_suggestions(
            "trying to reinvent 'safe concurrency' while almost throwing the baby with the bathwater, and making swift even more complex",
            ThrowBabyWithBathwater::default(),
            &[
                "trying to reinvent 'safe concurrency' while almost throwing the baby out with the bathwater, and making swift even more complex",
                "trying to reinvent 'safe concurrency' while almost throwing out the baby with the bathwater, and making swift even more complex",
            ],
            &[],
        );
    }

    #[test]
    fn fix_without_throwing_the_baby_with_the_bathwater() {
        assert_good_and_bad_suggestions(
            "Now, it's not that I can set min_resources to a larger value without throwing the baby with the bathwater.",
            ThrowBabyWithBathwater::default(),
            &[
                "Now, it's not that I can set min_resources to a larger value without throwing the baby out with the bathwater.",
                "Now, it's not that I can set min_resources to a larger value without throwing out the baby with the bathwater.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_seems_like_throwing_the_baby_with_the_bathwater() {
        assert_good_and_bad_suggestions(
            "that seems like throwing the baby with the bathwater",
            ThrowBabyWithBathwater::default(),
            &[
                "that seems like throwing the baby out with the bathwater",
                "that seems like throwing out the baby with the bathwater",
            ],
            &[],
        );
    }

    #[test]
    fn fix_throwing_away_the_baby_bathwater() {
        assert_suggestion_result(
            "Instead of gradually improving security, you are basically throwing away the baby with the bathwater.",
            ThrowBabyWithBathwater::default(),
            "Instead of gradually improving security, you are basically throwing out the baby with the bathwater.",
        );
    }

    #[test]
    fn fix_throw_away_the_baby_with_the_bath_water() {
        assert_suggestion_result(
            "then you don't have to throw away the baby with the bath water: you can apply the simple pattern optimization",
            ThrowBabyWithBathwater::default(),
            "then you don't have to throw out the baby with the bath water: you can apply the simple pattern optimization",
        );
    }

    #[test]
    fn fix_is_throwing_the_baby_with_the_bathwater() {
        assert_good_and_bad_suggestions(
            "Yeah, this is throwing the baby with the bathwater.",
            ThrowBabyWithBathwater::default(),
            &[
                "Yeah, this is throwing the baby out with the bathwater.",
                "Yeah, this is throwing out the baby with the bathwater.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_you_throw_away_the_baby_with_the_bathwater() {
        assert_suggestion_result(
            "Changes are that you throw away the baby with the bathwater.",
            ThrowBabyWithBathwater::default(),
            "Changes are that you throw out the baby with the bathwater.",
        );
    }

    #[test]
    fn fix_didnt_throw_the_baby_with_the_bathwater() {
        assert_good_and_bad_suggestions(
            "tried to remove all unnecessary things, I hope I didn't throw the baby with the bathwater.",
            ThrowBabyWithBathwater::default(),
            &[
                "tried to remove all unnecessary things, I hope I didn't throw the baby out with the bathwater.",
                "tried to remove all unnecessary things, I hope I didn't throw out the baby with the bathwater.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_feels_like_throwing_the_baby_with_the_bathwater() {
        assert_good_and_bad_suggestions(
            "That feels a bit like throwing the baby with the bathwater.",
            ThrowBabyWithBathwater::default(),
            &[
                "That feels a bit like throwing the baby out with the bathwater.",
                "That feels a bit like throwing out the baby with the bathwater.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_isnt_throwing_the_baby_with_the_bathwater() {
        assert_good_and_bad_suggestions(
            "OP isn't throwing the baby with the bathwater and he explains it very well in his post",
            ThrowBabyWithBathwater::default(),
            &[
                "OP isn't throwing the baby out with the bathwater and he explains it very well in his post",
                "OP isn't throwing out the baby with the bathwater and he explains it very well in his post",
            ],
            &[],
        );
    }

    #[test]
    fn fix_i_think_we_threw_baby_bath_water() {
        assert_good_and_bad_suggestions(
            "But I think we threw the baby with the bath water by getting rid of the stack traces completely.",
            ThrowBabyWithBathwater::default(),
            &[
                "But I think we threw the baby out with the bath water by getting rid of the stack traces completely.",
                "But I think we threw out the baby with the bath water by getting rid of the stack traces completely.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_but_we_threw_baby_bath_water() {
        assert_good_and_bad_suggestions(
            "I understand why the XML world of nonsense had to be stopped, but we threw the baby with the bath water.",
            ThrowBabyWithBathwater::default(),
            &[
                "I understand why the XML world of nonsense had to be stopped, but we threw the baby out with the bath water.",
                "I understand why the XML world of nonsense had to be stopped, but we threw out the baby with the bath water.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_thrown() {
        assert_good_and_bad_suggestions(
            "in such a way that they no longer work, we have thrown the baby with the bath water",
            ThrowBabyWithBathwater::default(),
            &[
                "in such a way that they no longer work, we have thrown the baby out with the bath water",
                "in such a way that they no longer work, we have thrown out the baby with the bath water",
            ],
            &[],
        );
    }

    #[test]
    fn fix_throws() {
        assert_good_and_bad_suggestions(
            "Throws the baby with the bathwater.",
            ThrowBabyWithBathwater::default(),
            &[
                "Throws the baby out with the bathwater.",
                "Throws out the baby with the bathwater.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_through_but_leave_misspelling() {
        assert_good_and_bad_suggestions(
            "I should not through the baby with the bathwater as they say.",
            ThrowBabyWithBathwater::default(),
            &[
                "I should not through the baby out with the bathwater as they say.",
                "I should not through out the baby with the bathwater as they say.",
            ],
            &[],
        );
    }

    // Don't flag intentional wordplay variants

    #[test]
    fn dont_flag_tossing() {
        assert_no_lints(
            "Give it a try before tossing the baby with the bath water.",
            ThrowBabyWithBathwater::default(),
        );
    }

    #[test]
    fn dont_flag_dumping() {
        assert_no_lints(
            "Jumping from there straight to vintage equipment amounts to dumping the baby with the bathwater, but modern digital cameras can be quite pricey",
            ThrowBabyWithBathwater::default(),
        );
    }

    #[test]
    fn dont_fix_parts() {
        assert_no_lints(
            "so I was wondering if I'm throwing parts of the baby with the bath water",
            ThrowBabyWithBathwater::default(),
        );
    }

    // Not handled as of yet

    #[test]
    #[ignore = "Missing `the` not implemented yet"]
    fn fix_missing_the() {
        assert_suggestion_result(
            "That's riding hype machine and throwing baby with bath water.",
            ThrowBabyWithBathwater::default(),
            "That's riding hype machine and throwing out the baby with bath water.",
        );
    }

    #[test]
    #[ignore = "Missing `bath` not implemented yet"]
    fn fix_water_instead_of_bathwater() {
        assert_suggestion_result(
            "My concern is that they have thrown the baby with the water.",
            ThrowBabyWithBathwater::default(),
            "My concern is that they have thrown out the baby with the water.",
        );
    }
}
