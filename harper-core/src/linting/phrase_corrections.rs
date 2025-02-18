use super::{Lint, LintKind, PatternLinter};
use crate::linting::Suggestion;
use crate::patterns::{ExactPhrase, Pattern, SimilarToPhrase};
use crate::{Token, TokenStringExt};

macro_rules! create_linter_map_phrase {
    ($name:ident, $pattern:expr, $correct_form:expr, $message:expr, $description:expr) => {
        #[doc = $description]
        pub struct $name {
            pattern: Box<dyn Pattern>,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    pattern: Box::new($pattern),
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl PatternLinter for $name {
            fn pattern(&self) -> &dyn Pattern {
                self.pattern.as_ref()
            }

            fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
                let span = matched_tokens.span().unwrap();
                let matched_text = span.get_content(source);

                Lint {
                    span,
                    lint_kind: LintKind::Miscellaneous,
                    suggestions: vec![Suggestion::replace_with_match_case(
                        $correct_form.chars().collect(),
                        matched_text,
                    )],
                    message: $message.to_string(),
                    priority: 31,
                }
            }

            fn description(&self) -> &'static str {
                $description
            }
        }
    };
}

/// Generate a linter that will look for a common phrase and correct mild errors that
/// are still composed of real words.
macro_rules! create_linter_for_phrase {
    ($name:ident, $correct_form:literal, $dist:literal) => {
        create_linter_map_phrase!(
            $name,
            SimilarToPhrase::from_phrase($correct_form, $dist),
            $correct_form,
            concat!("Did you mean the phrase `", $correct_form, "`?"),
            concat!(
                "Looks for slight improper modifications to the phrase `",
                $correct_form,
                "`."
            )
        );
    };
}

create_linter_for_phrase!(TurnItOff, "turn it off", 1);
create_linter_for_phrase!(HumanLife, "human life", 1);
create_linter_for_phrase!(ThatChallenged, "that challenged", 2);
create_linter_for_phrase!(NoLonger, "no longer", 1);
create_linter_for_phrase!(NeedHelp, "need help", 1);
create_linter_for_phrase!(OfCourse, "of course", 1);
create_linter_for_phrase!(AndAlike, "and alike", 1);
create_linter_for_phrase!(BadRap, "bad rap", 1);
create_linter_for_phrase!(BatedBreath, "bated breath", 1);
create_linter_for_phrase!(BeckAndCall, "beck and call", 1);
create_linter_for_phrase!(ChangeTack, "change tack", 1);
create_linter_for_phrase!(HungerPang, "hunger pang", 2);
create_linter_for_phrase!(EnMasse, "en masse", 1);
create_linter_for_phrase!(LetAlone, "let alone", 1);
create_linter_for_phrase!(SneakingSuspicion, "sneaking suspicion", 3);
create_linter_for_phrase!(SpecialAttention, "special attention", 1);
create_linter_for_phrase!(ThanOthers, "than others", 1);
create_linter_for_phrase!(SupposedTo, "supposed to", 1);
create_linter_for_phrase!(GetRidOf, "get rid of", 1);
create_linter_for_phrase!(GetsRidOf, "gets rid of", 1);
create_linter_for_phrase!(GettingRidOf, "getting rid of", 1);
create_linter_for_phrase!(GotRidOf, "got rid of", 1);
create_linter_for_phrase!(GottenRidOf, "gotten rid of", 1);

create_linter_map_phrase!(LoAndBehold, ExactPhrase::from_phrase("long and behold"), "lo and behold", "Did you mean `lo and behold`?", "Detects the exact phrase `long and behold` and suggests replacing it with the idiomatically correct `lo and behold`");

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::{
        BadRap, BatedBreath, ChangeTack, EnMasse, HungerPang, LetAlone, LoAndBehold, OfCourse,
        SneakingSuspicion, SpecialAttention, SupposedTo, ThanOthers, TurnItOff,
    };

    #[test]
    fn issue_574() {
        assert_lint_count("run by one", TurnItOff::default(), 0);
    }

    #[test]
    fn turn_it_off_clean_lower() {
        assert_lint_count("turn it off", TurnItOff::default(), 0);
    }

    #[test]
    fn turn_it_off_clean_upper() {
        assert_lint_count("Turn it off", TurnItOff::default(), 0);
    }

    #[test]
    fn of_confusion() {
        assert_suggestion_result("Turn it of", TurnItOff::default(), "Turn it off");
    }

    #[test]
    fn i_and_of_confusion() {
        assert_suggestion_result("Turn i of", TurnItOff::default(), "Turn it off");
    }

    #[test]
    fn off_course() {
        assert_suggestion_result(
            "Yes, off course we should do that.",
            OfCourse::default(),
            "Yes, of course we should do that.",
        );
    }

    #[test]
    fn o_course() {
        assert_suggestion_result(
            "Yes, o course we should do that.",
            OfCourse::default(),
            "Yes, of course we should do that.",
        );
    }

    #[test]
    fn bad_rep() {
        assert_suggestion_result("bad rep", BadRap::default(), "bad rap");
    }

    #[test]
    fn baited_breath() {
        assert_suggestion_result("baited breath", BatedBreath::default(), "bated breath");
    }

    #[test]
    fn change_tact() {
        assert_suggestion_result("change tact", ChangeTack::default(), "change tack");
    }

    #[test]
    fn hunger_pain() {
        assert_suggestion_result("hunger pain", HungerPang::default(), "hunger pang");
    }

    #[test]
    fn in_mass() {
        assert_suggestion_result("in mass", EnMasse::default(), "en masse");
    }

    #[test]
    fn let_along() {
        assert_suggestion_result("let along", LetAlone::default(), "let alone");
    }

    #[test]
    fn long_and_behold() {
        assert_suggestion_result("long and behold", LoAndBehold::default(), "lo and behold");
    }

    #[test]
    fn sneaky_suspicion() {
        assert_suggestion_result(
            "sneaky suspicion",
            SneakingSuspicion::default(),
            "sneaking suspicion",
        );
    }

    #[test]
    fn supposed_to() {
        assert_suggestion_result("suppose to", SupposedTo::default(), "supposed to");
    }

    #[test]
    fn get_rid_of() {
        assert_suggestion_result("Get rid off", GetRidOf::default(), "Get rid of");
    }

    #[test]
    fn gets_rid_of() {
        assert_suggestion_result("Gets rid off", GetRidOf::default(), "Gets rid of");
    }

    #[test]
    fn getting_rid_of() {
        assert_suggestion_result("Getting rid off", GetRidOf::default(), "Getting rid of");
    }

    #[test]
    fn got_rid_of() {
        assert_suggestion_result("Got rid off", GetRidOf::default(), "Got rid of");
    }

    #[test]
    fn gotten_rid_of() {
        assert_suggestion_result("Gotten rid off", GetRidOf::default(), "Gotten rid of");
    }

    #[test]
    fn spacial_attention() {
        assert_suggestion_result(
            "spacial attention",
            SpecialAttention::default(),
            "special attention",
        );
    }

    #[test]
    fn than_others() {
        assert_suggestion_result("Then others", ThanOthers::default(), "Than others");
    }

    #[test]
    fn now_on_hold() {
        assert_lint_count(
            "Those are now on hold for month.",
            LoAndBehold::default(),
            0,
        );
    }
}
