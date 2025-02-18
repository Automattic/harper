use crate::{
    patterns::{ExactPhrase, Pattern},
    Token, TokenStringExt,
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

macro_rules! create_closed_compound_linter {
    ($name:ident, $phrase:literal, $correct:expr) => {
        pub struct $name {
            pattern: Box<dyn Pattern>,
        }

        impl Default for $name {
            fn default() -> Self {
                let pattern = ExactPhrase::from_phrase($phrase);

                Self {
                    pattern: Box::new(pattern),
                }
            }
        }

        impl PatternLinter for $name {
            fn pattern(&self) -> &dyn Pattern {
                self.pattern.as_ref()
            }

            fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
                let span = matched_tokens.span()?;
                let orig_chars = span.get_content(source);

                Some(Lint {
                    span,
                    lint_kind: LintKind::WordChoice,
                    suggestions: vec![Suggestion::replace_with_match_case(
                        $correct.chars().collect(),
                        orig_chars,
                    )],
                    message: format!("Did you mean the closed compound `{}`?", $correct),
                    ..Default::default()
                })
            }

            fn description(&self) -> &'static str {
                concat!(
                    "Looks for incorrect spacing inside the closed compound `",
                    $correct,
                    "`."
                )
            }
        }
    };
}

create_closed_compound_linter!(Itself, "it self", "itself");
create_closed_compound_linter!(Myself, "my self", "myself");
create_closed_compound_linter!(Therefore, "there fore", "therefore");
create_closed_compound_linter!(Misunderstand, "miss understand", "misunderstand");
create_closed_compound_linter!(Misunderstood, "miss understood", "misunderstood");
create_closed_compound_linter!(Misuse, "miss use", "misuse");
create_closed_compound_linter!(Misused, "miss used", "misused");
create_closed_compound_linter!(Postpone, "post pone", "postpone");
create_closed_compound_linter!(Worldwide, "world wide", "worldwide");
create_closed_compound_linter!(Overall, "over all", "overall");
create_closed_compound_linter!(However, "how ever", "however");
create_closed_compound_linter!(Upset, "up set", "upset");
create_closed_compound_linter!(Intact, "in tact", "intact");
create_closed_compound_linter!(Somehow, "some how", "somehow");
create_closed_compound_linter!(Proofread, "proof read", "proofread");
create_closed_compound_linter!(Somebody, "some body", "somebody");
create_closed_compound_linter!(Anybody, "any body", "anybody");
create_closed_compound_linter!(Nothing, "no thing", "nothing");
create_closed_compound_linter!(Anywhere, "any where", "anywhere");
create_closed_compound_linter!(Instead, "in stead", "instead");
create_closed_compound_linter!(Somewhere, "some where", "somewhere");
create_closed_compound_linter!(Middleware, "middle ware", "middleware");
create_closed_compound_linter!(Into, "in to", "into");
create_closed_compound_linter!(Overclocking, "over clocking", "overclocking");
create_closed_compound_linter!(Backplane, "back plane", "backplane");
create_closed_compound_linter!(Overload, "over load", "overload");
create_closed_compound_linter!(Underclock, "under clock", "underclock");
create_closed_compound_linter!(Devops, "dev ops", "devops");
create_closed_compound_linter!(Multithreading, "multi threading", "multithreading");
create_closed_compound_linter!(Everywhere, "every where", "everywhere");
create_closed_compound_linter!(Multicore, "multi core", "multicore");
create_closed_compound_linter!(Multimedia, "multi media", "multimedia");
create_closed_compound_linter!(Widespread, "wide spread", "widespread");
create_closed_compound_linter!(Notwithstanding, "not with standing", "notwithstanding");
create_closed_compound_linter!(Anyhow, "any how", "anyhow");
create_closed_compound_linter!(Nonetheless, "none the less", "nonetheless");
create_closed_compound_linter!(Thereupon, "there upon", "thereupon");
create_closed_compound_linter!(Forthwith, "forth with", "forthwith");
create_closed_compound_linter!(Insofar, "in so far", "insofar");
create_closed_compound_linter!(Whereupon, "where upon", "whereupon");
create_closed_compound_linter!(Upward, "up ward", "upward");
create_closed_compound_linter!(Henceforth, "hence forth", "henceforth");
create_closed_compound_linter!(Regardless, "regard less", "regardless");
create_closed_compound_linter!(Overnight, "over night", "overnight");
create_closed_compound_linter!(Furthermore, "further more", "furthermore");
create_closed_compound_linter!(Laptop, "lap top", "laptop");
create_closed_compound_linter!(Desktop, "desk top", "desktop");

#[cfg(test)]
mod tests {
    use super::{
        Anyhow, Forthwith, Henceforth, Insofar, Nonetheless, Notwithstanding, Overnight,
        Regardless, Thereupon, Upward, Whereupon, Widespread,
    };
    use super::{
        However, Itself, Misunderstood, Misuse, Misused, Myself, Overall, Therefore, Worldwide,
    };
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn it_self() {
        let test_sentence = "The project, it self, was quite challenging.";
        let expected = "The project, itself, was quite challenging.";
        assert_suggestion_result(test_sentence, Itself::default(), expected);
    }

    #[test]
    fn my_self() {
        let test_sentence = "He treated my self with respect.";
        let expected = "He treated myself with respect.";
        assert_suggestion_result(test_sentence, Myself::default(), expected);
    }

    #[test]
    fn there_fore() {
        let test_sentence = "This is the reason; there fore, this is true.";
        let expected = "This is the reason; therefore, this is true.";
        assert_suggestion_result(test_sentence, Therefore::default(), expected);
    }

    #[test]
    fn mis_understood() {
        let test_sentence = "She miss understood the instructions.";
        let expected = "She misunderstood the instructions.";
        assert_suggestion_result(test_sentence, Misunderstood::default(), expected);
    }

    #[test]
    fn mis_use() {
        let test_sentence = "He tends to miss use the tool.";
        let expected = "He tends to misuse the tool.";
        assert_suggestion_result(test_sentence, Misuse::default(), expected);
    }

    #[test]
    fn mis_used() {
        let test_sentence = "The software was miss used.";
        let expected = "The software was misused.";
        assert_suggestion_result(test_sentence, Misused::default(), expected);
    }

    #[test]
    fn world_wide() {
        let test_sentence = "The world wide impact was significant.";
        let expected = "The worldwide impact was significant.";
        assert_suggestion_result(test_sentence, Worldwide::default(), expected);
    }

    #[test]
    fn over_all() {
        let test_sentence = "The over all performance was good.";
        let expected = "The overall performance was good.";
        assert_suggestion_result(test_sentence, Overall::default(), expected);
    }

    #[test]
    fn how_ever() {
        let test_sentence = "This is true, how ever, details matter.";
        let expected = "This is true, however, details matter.";
        assert_suggestion_result(test_sentence, However::default(), expected);
    }

    #[test]
    fn wide_spread() {
        let test_sentence = "The news was wide spread throughout the region.";
        let expected = "The news was widespread throughout the region.";
        assert_suggestion_result(test_sentence, Widespread::default(), expected);
    }

    #[test]
    fn not_with_standing() {
        let test_sentence = "They decided to proceed not with standing any further delay.";
        let expected = "They decided to proceed notwithstanding any further delay.";
        assert_suggestion_result(test_sentence, Notwithstanding::default(), expected);
    }

    #[test]
    fn any_how() {
        let test_sentence = "She solved the problem any how, even under pressure.";
        let expected = "She solved the problem anyhow, even under pressure.";
        assert_suggestion_result(test_sentence, Anyhow::default(), expected);
    }

    #[test]
    fn none_the_less() {
        let test_sentence = "The results were disappointing, none the less, they continued.";
        let expected = "The results were disappointing, nonetheless, they continued.";
        assert_suggestion_result(test_sentence, Nonetheless::default(), expected);
    }

    #[test]
    fn there_upon() {
        let test_sentence = "A decision was made there upon reviewing the data.";
        let expected = "A decision was made thereupon reviewing the data.";
        assert_suggestion_result(test_sentence, Thereupon::default(), expected);
    }

    #[test]
    fn forth_with() {
        let test_sentence = "Please reply forth with to our previous inquiry.";
        let expected = "Please reply forthwith to our previous inquiry.";
        assert_suggestion_result(test_sentence, Forthwith::default(), expected);
    }

    #[test]
    fn in_so_far() {
        let test_sentence = "This rule applies in so far as it covers all cases.";
        let expected = "This rule applies insofar as it covers all cases.";
        assert_suggestion_result(test_sentence, Insofar::default(), expected);
    }

    #[test]
    fn where_upon() {
        let test_sentence = "They acted where upon the circumstances allowed.";
        let expected = "They acted whereupon the circumstances allowed.";
        assert_suggestion_result(test_sentence, Whereupon::default(), expected);
    }

    #[test]
    fn up_ward() {
        let test_sentence = "The temperature moved up ward during the afternoon.";
        let expected = "The temperature moved upward during the afternoon.";
        assert_suggestion_result(test_sentence, Upward::default(), expected);
    }

    #[test]
    fn hence_forth() {
        let test_sentence = "All new policies apply hence forth immediately.";
        let expected = "All new policies apply henceforth immediately.";
        assert_suggestion_result(test_sentence, Henceforth::default(), expected);
    }

    #[test]
    fn regard_less() {
        let test_sentence = "The decision was made, regard less of the opposition.";
        let expected = "The decision was made, regardless of the opposition.";
        assert_suggestion_result(test_sentence, Regardless::default(), expected);
    }

    #[test]
    fn over_night() {
        let test_sentence = "They set off on their journey over night.";
        let expected = "They set off on their journey overnight.";
        assert_suggestion_result(test_sentence, Overnight::default(), expected);
    }
}
