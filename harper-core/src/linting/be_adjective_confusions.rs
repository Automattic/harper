use crate::{
    CharStringExt, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{
        ExprLinter, Lint, LintGroup, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_hyphen, followed_by_word, preceded_by_word},
    },
    patterns::{Word, WordSet},
};

/// A linter that handles "be + verb → be + adjective" confusions
struct BeAdjectiveLinter {
    expr: All,
    verb: &'static str,
    adjective: &'static str,
    not_if_followed_by: &'static [&'static str],
    message: &'static str,
}

impl BeAdjectiveLinter {
    fn new(
        verb: &'static str,
        adjective: &'static str,
        exception_words: &'static [&'static str],
        message: &'static str,
    ) -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_any_of(vec![
                    Box::new(
                        SequenceExpr::default()
                            .then_subject_pronoun()
                            .t_ws()
                            .t_set(&["am", "are", "is", "was", "were"]),
                    ),
                    Box::new(WordSet::new(&[
                        // Correct contractions
                        "i'm", "we're", "you're", "he's", "she's", "they're",
                        // Incorrect contractions missing their apostrophes that should not cause problems
                        "im", "theyre",
                        // Note that "were" is already included as a form of "be" above
                    ])),
                ])
                .t_ws()
                .t_aco(verb)
                // So far we prohibit "it" as a possible subject.
                // "it is worry/concern" is often grammatically legitimate
                // and would cause false positives.
                // Future improvements could add "it" support with
                // more sophisticated context analysis.
                .and_not(Word::new("it")),
            verb,
            adjective,
            message,
            not_if_followed_by: exception_words,
        }
    }
}

impl ExprLinter for BeAdjectiveLinter {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let wtok = toks.last()?;

        if followed_by_hyphen(ctx)
            || preceded_by_word(ctx, |w| w.get_ch(src).eq_str("there"))
            || followed_by_word(ctx, |w| {
                w.get_ch(src)
                    .eq_any_ignore_ascii_case_str(self.not_if_followed_by)
            })
        {
            return None;
        }

        Some(Lint {
            span: wtok.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                self.adjective,
                wtok.span.get_content(src),
            )],
            message: self.message.to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects incorrect use of 'be + verb' instead of `be + adjective`."
    }
}

pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::empty();

    macro_rules! add_be_adjective_mappings {
        ($group:expr, { $($name:expr => ($verb:expr, $adjective:expr, $exceptions:expr, $message:expr)),+ $(,)? }) => {
            $(
                $group.add(
                    $name,
                    Box::new(BeAdjectiveLinter::new($verb, $adjective, $exceptions, $message)),
                );
            )+
        };
    }

    add_be_adjective_mappings!(group, {
        "BeBiased" => ("bias", "biased", &["adjusted", "corrected", "correcting", "field", "free", "values"], "Use the adjective 'biased' instead of the noun 'bias'."),
        "BeConcerned" => ("concern", "concerned", &[], "Use the adjective 'concerned' instead of the noun 'concern'."),
        "BePrejudiced" => ("prejudice", "prejudiced", &[], "Use the adjective 'prejudiced' instead of the noun 'prejudice'."),
        "BeShocked" => ("shock", "shocked", &["absorbers", "jocks", "resistant", "sensitive"], "Use the adjective 'shocked' instead of the noun 'shock'."),
        "BeWorried" => ("worry", "worried", &["free", "wart", "warts"], "Use the adjective 'worried' instead of the verb 'worry'."),
    });

    group.set_all_rules_to(Some(true));
    group
}

#[cfg(test)]
mod tests {
    mod biased {
        use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

        use super::super::lint_group;

        #[test]
        fn he_s_bias() {
            assert_suggestion_result(
                "he's just on a rush to write a rejection letter probably because he's bias toward the origin or affiliation of the author",
                lint_group(),
                "he's just on a rush to write a rejection letter probably because he's biased toward the origin or affiliation of the author",
            );
        }

        #[test]
        fn i_am_bias() {
            assert_suggestion_result(
                "I guess I am bias/prefer udd-style fragments",
                lint_group(),
                "I guess I am biased/prefer udd-style fragments",
            );
        }

        #[test]
        fn i_m_bias() {
            assert_suggestion_result(
                "I'm bias towards vunit as it's by far the most user friendly HDL test runner I know of.",
                lint_group(),
                "I'm biased towards vunit as it's by far the most user friendly HDL test runner I know of.",
            );
        }

        #[test]
        fn im_bias() {
            assert_suggestion_result(
                "im bias, cause thats all im here for",
                lint_group(),
                "im biased, cause thats all im here for",
            );
        }

        #[test]
        fn they_re_bias() {
            assert_suggestion_result(
                "See, I told you they’re bias because they flagged my comment!",
                lint_group(),
                "See, I told you they’re biased because they flagged my comment!",
            );
        }

        // This one could also be corrected by a they're->their rule: "to admit their bias upfront"
        #[test]
        fn they_re_bias_could_also_be_their_bias() {
            assert_suggestion_result(
                "As a general rule I try to steer clear of the major news media outlets because of their outright refusal to admit they're bias upfront.",
                lint_group(),
                "As a general rule I try to steer clear of the major news media outlets because of their outright refusal to admit they're biased upfront.",
            );
        }

        #[test]
        fn were_bias() {
            assert_suggestion_result(
                "the generated sample were bias in presence of negative cross-section",
                lint_group(),
                "the generated sample were biased in presence of negative cross-section",
            );
        }

        // Negative tests for "bias"

        #[test]
        fn dont_flag_bias_hyphen_corrected() {
            assert_no_lints("I gather they are bias-corrected estimates.", lint_group());
        }

        #[test]
        fn dont_flag_bias_space_corrected() {
            assert_no_lints(
                "All the images were bias corrected using N4 algorithm with a threshold value of 0.001.",
                lint_group(),
            );
        }

        #[test]
        fn dont_flag_its_bias_free() {
            assert_no_lints(
                "Both the original model, and its bias-free counterpart, will be loaded.",
                lint_group(),
            );
        }

        #[test]
        fn dont_flag_if_there_were_bias() {
            assert_no_lints(
                "which depend on several variables: if there were bias, the average size of the classes, as well as the number of teachers at the school",
                lint_group(),
            );
        }
    }

    mod concerned {
        use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

        use super::super::lint_group;

        #[test]
        fn hes_concern() {
            assert_suggestion_result(
                "He's concern about my mental health because I'm alone in this city",
                lint_group(),
                "He's concerned about my mental health because I'm alone in this city",
            );
        }

        #[test]
        fn i_am_concern() {
            assert_suggestion_result(
                "I am learning to use this, but I am concern on data privacy and sharing issues.",
                lint_group(),
                "I am learning to use this, but I am concerned on data privacy and sharing issues.",
            );
        }

        #[test]
        fn im_concern() {
            assert_suggestion_result(
                "I'm concern if there is missing CVEs.",
                lint_group(),
                "I'm concerned if there is missing CVEs.",
            );
        }

        #[test]
        fn im_no_apostrophe_concern() {
            assert_suggestion_result(
                "The only thing im concern about is the crash.",
                lint_group(),
                "The only thing im concerned about is the crash.",
            );
        }

        #[test]
        #[ignore = "`Its` is currently not flagged because it's not always an error."]
        fn its_no_apostrophe_concern() {
            assert_suggestion_result(
                "yes no one there answered and I thought its concern with this repo too so I clone it here!",
                lint_group(),
                "yes no one there answered and I thought its concerned with this repo too so I cloned it here!",
            );
        }

        #[test]
        fn shes_concern() {
            assert_suggestion_result(
                "she often courier food to her 40 year old son living in Jeju island because she’s concern about her son’s well being",
                lint_group(),
                "she often courier food to her 40 year old son living in Jeju island because she’s concerned about her son’s well being",
            );
        }

        #[test]
        fn were_concern() {
            assert_suggestion_result(
                "As far as we're concern, this is a very simple try catch on your end",
                lint_group(),
                "As far as we're concerned, this is a very simple try catch on your end",
            );
        }

        #[test]
        fn we_are_concern() {
            assert_suggestion_result(
                "We are concern and prioritize users' feedback and maintainability.",
                lint_group(),
                "We are concerned and prioritize users' feedback and maintainability.",
            );
        }

        #[test]
        fn you_are_concern() {
            assert_suggestion_result(
                "you are concern only in logging a more appropriate message error, but continue to throw the opaque assertion error.",
                lint_group(),
                "you are concerned only in logging a more appropriate message error, but continue to throw the opaque assertion error.",
            );
        }

        // Negative tests for "concern"

        #[test]
        fn dont_flag_noun_concern() {
            assert_no_lints(
                "My main concern is the data that could be lost using this deletion.",
                lint_group(),
            );
        }

        #[test]
        fn dont_flag_verb_concern() {
            assert_no_lints("This doesn't concern me at all.", lint_group());
        }

        #[test]
        fn dont_flag_concern_free() {
            assert_no_lints("They don't pretend they're concern-free.", lint_group());
        }

        #[test]
        fn dont_flag_legit_its_concern() {
            assert_no_lints(
                "The ByteBlockPool loses its reference to the buffers it's just zeroed, so it seems outside its concern to zero the buffers.",
                lint_group(),
            );
        }

        #[test]
        #[ignore = "This is a different error that should be caught by `ShouldContract`"]
        fn dont_flag_youre_your_error() {
            assert_no_lints(
                "You're concern then is literally the \"-like\" in the message?",
                lint_group(),
            );
        }
    }

    mod prejudiced {
        use crate::linting::tests::assert_suggestion_result;

        use super::super::lint_group;

        #[test]
        fn i_was_prejudice() {
            assert_suggestion_result(
                "I didn't know I was prejudice against some of my own people until I went looking for web framework documentation and was met with a BLM banner.",
                lint_group(),
                "I didn't know I was prejudiced against some of my own people until I went looking for web framework documentation and was met with a BLM banner.",
            );
        }

        #[test]
        fn shes_prejudice() {
            assert_suggestion_result(
                "She's prejudice against muslims.",
                lint_group(),
                "She's prejudiced against muslims.",
            );
        }

        #[test]
        fn theyre_prejudice() {
            assert_suggestion_result(
                "The people who manage the law are all about lying and tricking people, theyre prejudice and trying to make people look stupid",
                lint_group(),
                "The people who manage the law are all about lying and tricking people, theyre prejudiced and trying to make people look stupid",
            );
        }
    }

    mod shocked {
        use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

        use super::super::lint_group;

        #[test]
        fn we_were_shock() {
            assert_suggestion_result(
                "We were shock because its working fine on 1st of April just after 3 days its not working with the same settings, that is our major concern.",
                lint_group(),
                "We were shocked because its working fine on 1st of April just after 3 days its not working with the same settings, that is our major concern.",
            );
        }

        // Negative tests for "shock"

        #[test]
        fn dont_flag_shock_jocks() {
            assert_no_lints(
                "They're not commentators/anaylsts, they're shock jocks.",
                lint_group(),
            );
        }

        #[test]
        #[ignore = "Legitimate use of noun that the linter cannot yet distinguish."]
        fn dont_flag_were_shock() {
            assert_no_lints(
                "My first reactions when the topic of cryonics came up (early in our relationship) were shock, a bit of revulsion, and a lot of confusion.",
                lint_group(),
            );
        }
    }

    mod worried {
        use crate::linting::tests::{
            assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
        };

        use super::super::lint_group;

        #[test]
        fn he_is_worry() {
            assert_suggestion_result(
                "I guess he is worry about \" * user * \" tag.",
                lint_group(),
                "I guess he is worried about \" * user * \" tag.",
            );
        }

        #[test]
        fn he_was() {
            assert_suggestion_result(
                "So he was worry about her. Especially, when he got no response by calling her on her phone nor ranging her doorbell.",
                lint_group(),
                "So he was worried about her. Especially, when he got no response by calling her on her phone nor ranging her doorbell.",
            );
        }

        #[test]
        fn i_am_worry() {
            assert_suggestion_result(
                "I didn't see any section dedicated to this so I am worry about:",
                lint_group(),
                "I didn't see any section dedicated to this so I am worried about:",
            );
        }

        #[test]
        fn im_worry() {
            assert_suggestion_result(
                "but I'm worry about memory leak caused by that long delay",
                lint_group(),
                "but I'm worried about memory leak caused by that long delay",
            );
        }

        #[test]
        fn im_no_apostrophe_worry() {
            assert_suggestion_result(
                "im worry now that if this inbound was dangerous and maybe it does something to my pc",
                lint_group(),
                "im worried now that if this inbound was dangerous and maybe it does something to my pc",
            );
        }

        #[test]
        fn i_was() {
            assert_suggestion_result(
                "So that's why I was worry.",
                lint_group(),
                "So that's why I was worried.",
            );
        }

        #[test]
        fn i_were() {
            assert_suggestion_result(
                "The only things that I were worry about is the data that could be lost using this deletion.",
                lint_group(),
                "The only things that I were worried about is the data that could be lost using this deletion.",
            );
        }

        #[test]
        fn they_are_worry() {
            assert_suggestion_result(
                "at the same time they are worry about the price for the upgrade each 3 years",
                lint_group(),
                "at the same time they are worried about the price for the upgrade each 3 years",
            );
        }

        #[test]
        fn theyre_worry() {
            assert_suggestion_result(
                "Because they're worry this link is spam or they scare have to pay more money.",
                lint_group(),
                "Because they're worried this link is spam or they scare have to pay more money.",
            );
        }

        #[test]
        fn we_are() {
            assert_suggestion_result(
                "We are analised this and we are worry because when our platform go to market",
                lint_group(),
                "We are analised this and we are worried because when our platform go to market",
            );
        }

        #[test]
        fn were() {
            assert_suggestion_result(
                "We're worry about all kinds of minority representation in TV.",
                lint_group(),
                "We're worried about all kinds of minority representation in TV.",
            );
        }

        #[test]
        fn you_are() {
            assert_suggestion_result(
                "You are worry because we are not annotating view interface itself, right?",
                lint_group(),
                "You are worried because we are not annotating view interface itself, right?",
            );
        }

        #[test]
        fn youre() {
            assert_suggestion_result(
                "You're worry about memory usage and wanna be sure that a Sequence-class won't hold your activity against GC — declare this class as static",
                lint_group(),
                "You're worried about memory usage and wanna be sure that a Sequence-class won't hold your activity against GC — declare this class as static",
            );
        }

        // Negative tests for "worry"

        #[test]
        fn dont_flag_it_is() {
            assert_no_lints(
                "Part of it is worry that my bosses will get angry and fire me.",
                lint_group(),
            );
        }

        #[test]
        fn dont_flag_it_was() {
            assert_no_lints(
                "Because what followed wasn't indifference, it was worry.",
                lint_group(),
            );
        }

        #[test]
        fn dont_flag_she_was_worry_free() {
            assert_no_lints("Finally, she was worry-free.", lint_group());
        }

        #[test]
        fn dont_flag_theyre_worry_free() {
            assert_no_lints("They don't pretend they're worry-free.", lint_group());
        }

        #[test]
        fn dont_flag_worry_warts() {
            assert_no_lints("They’re worry warts", lint_group());
        }

        #[test]
        fn dont_flag_were_worry_space_free() {
            assert_no_lints(
                "Thanks to jQuery, we're worry free from browser compatibility.",
                lint_group(),
            );
        }

        #[test]
        #[ignore = "different error case not handled by this linter"]
        fn different_case_not_handled() {
            assert_good_and_bad_suggestions(
                "Myself along with others are using it on an iPad successfully, so it is worry to hear that is broken for you.",
                lint_group(),
                &[
                    "Myself along with others are using it on an iPad successfully, so it is worrying to hear that is broken for you.",
                    "Myself along with others are using it on an iPad successfully, so it is a worry to hear that is broken for you.",
                ],
                &[],
            );
        }

        #[test]
        #[ignore = "This is a different error that should be caught by `ShouldContract`"]
        fn dont_flag_youre_youre_error() {
            assert_no_lints("I think you're worry is not really an issue.", lint_group());
        }
    }
}
