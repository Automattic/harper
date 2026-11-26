mod go_the_the;
mod repeated_words_general;
mod the_the_to_that_the;

use super::merge_linters::merge_linters;
use go_the_the::GoTheThe;
use repeated_words_general::RepeatedWordsGeneral;
use the_the_to_that_the::TheTheToThatThe;

merge_linters!(
    RepeatedWords =>
        GoTheThe,
        RepeatedWordsGeneral,
        TheTheToThatThe
    => "This rule looks for repetitions of words that are not homographs."
);

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
    };

    use super::{super::tests::assert_lint_count, GoTheThe, RepeatedWordsGeneral, TheTheToThatThe};

    // General

    #[test]
    fn catches_basic() {
        assert_lint_count(
            "I wanted the the banana.",
            RepeatedWordsGeneral::default(),
            1,
        )
    }

    #[test]
    fn does_not_lint_homographs_address() {
        assert_no_lints(
            "To address address problems.",
            RepeatedWordsGeneral::default(),
        );
    }

    #[test]
    fn does_not_lint_homographs_record() {
        assert_no_lints("To record record profits.", RepeatedWordsGeneral::default());
    }

    #[test]
    fn issue_253() {
        assert_lint_count(
            "this paper shows that, while the method may be more accurate accurate, the turnout overestimate suggests that self-selection bias is not sufficiently reduced",
            RepeatedWordsGeneral::default(),
            1,
        );
    }

    #[test]
    fn issue_333() {
        assert_suggestion_result(
            "This is is a test",
            RepeatedWordsGeneral::default(),
            "This is a test",
        );
    }

    #[test]
    fn double_a() {
        assert_suggestion_result(
            "This is a a test",
            RepeatedWordsGeneral::default(),
            "This is a test",
        );
    }

    #[test]
    fn double_and() {
        assert_suggestion_result(
            "And and this is also a test",
            RepeatedWordsGeneral::default(),
            "And this is also a test",
        );
    }

    #[test]
    fn on_on_github() {
        assert_suggestion_result(
            "Take a look at the project on on GitHub.",
            RepeatedWordsGeneral::default(),
            "Take a look at the project on GitHub.",
        );
    }

    #[test]
    fn as_as() {
        assert_suggestion_result(
            "he is as as hard as nails",
            RepeatedWordsGeneral::default(),
            "he is as hard as nails",
        );
    }

    #[test]
    fn dont_flag_first_hyphenated() {
        assert_no_lints(
            "The driver-facing camera and microphone are only logged if you explicitly opt-in in settings.",
            RepeatedWordsGeneral::default(),
        );
    }

    #[test]
    fn dont_flag_hyphenated_either_side() {
        assert_no_lints("foo-foo foo bar bar-bar", RepeatedWordsGeneral::default());
    }

    #[test]
    fn dont_flag_repeated_laughter() {
        for source in ["ha ha", "Ha ha", "hah hah", "ha ha ha"] {
            assert_lint_count(source, RepeatedWordsGeneral::default(), 0);
        }
    }

    // TheTheToThatThe (contrived tests)

    #[test]
    fn dont_flag_simple_the_the() {
        assert_no_lints("this is the the simple case", TheTheToThatThe::default());
    }

    #[test]
    fn fix_so_that_the() {
        assert_suggestion_result(
            "this is so the the other case is tested",
            TheTheToThatThe::default(),
            "this is so that the other case is tested",
        );
    }

    // TheTheToThatThe (real world tests)

    #[test]
    fn fact_the_the_to_that() {
        assert_suggestion_result(
            "This PR fixes the fact the the / page and some of the other new pages are reverting to the browser font",
            TheTheToThatThe::default(),
            "This PR fixes the fact that the / page and some of the other new pages are reverting to the browser font",
        );
    }

    #[test]
    fn found_the_the_just_double() {
        assert_suggestion_result(
            "and I've never found the the dashboard feed materially useful",
            TheTheToThatThe::default(),
            "and I've never found the dashboard feed materially useful",
        );
    }

    #[test]
    fn found_the_the_could_be_either() {
        assert_good_and_bad_suggestions(
            "I found the the shape of feature from the PillarFeatureNet becomes [64] instead of [N, 64]",
            TheTheToThatThe::default(),
            &[
                "I found that the shape of feature from the PillarFeatureNet becomes [64] instead of [N, 64]",
                "I found the shape of feature from the PillarFeatureNet becomes [64] instead of [N, 64]",
            ],
            &[],
        );
    }

    #[test]
    fn found_the_the_to_that() {
        assert_suggestion_result(
            "I was directed to the MongoDB website and found the the newer versions require you to have the mongosh shell",
            TheTheToThatThe::default(),
            "I was directed to the MongoDB website and found that the newer versions require you to have the mongosh shell",
        );
    }

    #[test]
    fn say_the_the_to_that() {
        assert_suggestion_result(
            "So, I'm going to say the the shown output is a bug in using Avro",
            TheTheToThatThe::default(),
            "So, I'm going to say that the shown output is a bug in using Avro",
        );
    }

    #[test]
    fn show_the_the_just_double() {
        assert_suggestion_result(
            "It would be better to show the the first logical line (i.e. the first paragraph).",
            TheTheToThatThe::default(),
            "It would be better to show the first logical line (i.e. the first paragraph).",
        );
    }

    #[test]
    fn so_the_the_could_be_either() {
        assert_good_and_bad_suggestions(
            "accordion-body doesn't have data- attributes to read from, so the The proper solution isn't quite working",
            TheTheToThatThe::default(),
            &[
                // TODO: replace_with_match_cases goes by index so results in `thaT` instead of `that`
                // "accordion-body doesn't have data- attributes to read from, so that the proper solution isn't quite working",
                "accordion-body doesn't have data- attributes to read from, so the proper solution isn't quite working",
            ],
            &[],
        );
    }

    #[test]
    fn so_the_the_just_double() {
        assert_suggestion_result(
            "So the the choice is really between pursuing a compromise ...",
            TheTheToThatThe::default(),
            "So the choice is really between pursuing a compromise ...",
        );
    }

    #[test]
    fn think_the_the_to_that() {
        assert_suggestion_result(
            "I don't think the the additional flask route was the issue because I got your example to work.",
            TheTheToThatThe::default(),
            "I don't think that the additional flask route was the issue because I got your example to work.",
        );
    }

    #[test]
    fn think_the_the_could_be_either() {
        assert_good_and_bad_suggestions(
            "but I think the the lack of transparency and attribution to the original coder makes this much more an act of opportunism",
            TheTheToThatThe::default(),
            &[
                "but I think that the lack of transparency and attribution to the original coder makes this much more an act of opportunism",
                "but I think the lack of transparency and attribution to the original coder makes this much more an act of opportunism",
            ],
            &[],
        );
    }

    // GoTheThe (real world tests)

    #[test]
    fn go_the_the() {
        assert_suggestion_result(
            "Go the the machine where slave is running.",
            GoTheThe::default(),
            "Go to the machine where slave is running.",
        );
    }

    #[test]
    fn going_the_the() {
        assert_suggestion_result(
            "Start by going the the directory where you want your binaries and run.",
            GoTheThe::default(),
            "Start by going to the directory where you want your binaries and run.",
        );
    }

    #[test]
    fn gone_the_the() {
        assert_suggestion_result(
            "Prof's have gone the the extent of saying, \"I won't waste time showing you this, you could just go on youtube\".",
            GoTheThe::default(),
            "Prof's have gone to the extent of saying, \"I won't waste time showing you this, you could just go on youtube\".",
        );
    }

    #[test]
    fn goes_the_the() {
        assert_suggestion_result(
            "When you install a plugin from CLI it goes the the ~/.copilot/installed-plugins folder",
            GoTheThe::default(),
            "When you install a plugin from CLI it goes to the ~/.copilot/installed-plugins folder",
        );
    }

    #[test]
    fn went_the_the() {
        assert_suggestion_result(
            "Went the the WiFi screen and clicked save as fast as possible for 30 seconds.",
            GoTheThe::default(),
            "Went to the WiFi screen and clicked save as fast as possible for 30 seconds.",
        );
    }
}
