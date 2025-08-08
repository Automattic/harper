mod to_too;
mod too_to;

use super::merge_linters::merge_linters;
use super::{ExprLinter, Lint, LintKind, Suggestion};
use to_too::ToToo;
use too_to::TooTo;

merge_linters!(ToTwoToo => ToToo, TooTo => "fasd");

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    use super::ToTwoToo;

    #[test]
    fn fixes_to_late() {
        assert_suggestion_result(
            "You're 7 years to late",
            ToTwoToo::default(),
            "You're 7 years too late",
        );
    }

    #[test]
    fn fixes_to_ambitious() {
        assert_suggestion_result(
            "The project scope is to ambitious",
            ToTwoToo::default(),
            "The project scope is too ambitious",
        );
    }

    #[test]
    fn fixes_end_of_sent() {
        assert_suggestion_result(
            "She wants ice cream, to.",
            ToTwoToo::default(),
            "She wants ice cream, too.",
        );

        // Should work regardless of comma
        assert_suggestion_result(
            "She wants ice cream to.",
            ToTwoToo::default(),
            "She wants ice cream too.",
        );
    }

    #[test]
    fn flags_to_hungry() {
        assert_lint_count("I am to hungry.", ToTwoToo::default(), 1);
    }

    #[test]
    fn no_lint_on_proper_too() {
        assert_no_lints("I am too hungry.", ToTwoToo::default());
    }

    #[test]
    fn flags_to_quickly() {
        assert_lint_count(
            "They moved to quickly across the field.",
            ToTwoToo::default(),
            1,
        );
    }

    #[test]
    fn flags_to_with_irregular_whitespace() {
        assert_lint_count("She was to\t   tired.", ToTwoToo::default(), 1);
        assert_lint_count("He felt it was\nto cold.", ToTwoToo::default(), 1);
    }

    #[test]
    fn flags_to_with_trailing_punct() {
        assert_lint_count("He spoke to loud!", ToTwoToo::default(), 1);
        assert_lint_count("He spoke to loud?", ToTwoToo::default(), 1);
        assert_lint_count("He spoke to loud.", ToTwoToo::default(), 1);
    }

    #[test]
    fn no_lint_to_eat() {
        assert_no_lints(
            "Please remember to eat your vegetables.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_to_nashville_or_you() {
        assert_no_lints("I’m going to Nashville next week.", ToTwoToo::default());
        assert_no_lints("Talk to you later.", ToTwoToo::default());
    }

    #[test]
    fn fixes_too_go() {
        assert_suggestion_result(
            "I want too go abroad.",
            ToTwoToo::default(),
            "I want to go abroad.",
        );
    }

    #[test]
    fn fixes_too_him() {
        assert_suggestion_result(
            "Give it too him as a gift",
            ToTwoToo::default(),
            "Give it to him as a gift",
        );
    }

    #[test]
    fn fixes_too_the() {
        assert_suggestion_result(
            "We're going too the conference.",
            ToTwoToo::default(),
            "We're going to the conference.",
        );
    }

    #[test]
    fn fixes_too_a() {
        assert_suggestion_result(
            "We're going too a concert.",
            ToTwoToo::default(),
            "We're going to a concert.",
        );
    }

    #[test]
    fn fixes_to_hard() {
        assert_suggestion_result(
            "It's not to hard, is it?",
            ToTwoToo::default(),
            "It's not too hard, is it?",
        );
    }

    #[test]
    fn fixes_to_easy() {
        assert_suggestion_result(
            "It's not to easy, is it?",
            ToTwoToo::default(),
            "It's not too easy, is it?",
        );
    }

    #[test]
    fn no_lint_too_hot() {
        assert_no_lints("The coffee is too hot to drink.", ToTwoToo::default());
    }

    #[test]
    fn no_lint_too_loud() {
        assert_no_lints(
            "The music was too loud, making it hard to hear.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_too_shy() {
        assert_no_lints("He's too shy to speak in public.", ToTwoToo::default());
    }

    #[test]
    fn no_lint_too_sweet() {
        assert_no_lints("The cake is too sweet for my taste.", ToTwoToo::default());
    }

    #[test]
    fn no_lint_too_expensive() {
        assert_no_lints(
            "It's too expensive for me to buy right now.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_too_hard() {
        assert_no_lints(
            "She worked too hard and ended up getting sick.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_too_complicated() {
        assert_no_lints(
            "The instructions were too complicated to understand.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_too_too() {
        assert_no_lints(
            "I like apples, and my brother does too.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_too_too_2() {
        assert_no_lints(
            "She's coming to the party, and he is too.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_too_too_3() {
        assert_no_lints(
            "I want to go to the beach, and you do too?",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_too_too_4() {
        assert_no_lints(
            "He's a talented musician, and a great friend too.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_too_too_5() {
        assert_no_lints(
            "The movie was good, and the popcorn was delicious too.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_too_difficult_too_close() {
        assert_no_lints(
            "The problem is too difficult, and the deadline is too close.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_too_good_too_nice() {
        assert_no_lints(
            "He's too good at the game, and he's too nice to win.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn allow_young_musicians() {
        assert_no_lints(
            "Bringing Hope and Opportunity to Young Musicians",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn allow_semicolon() {
        assert_no_lints("Attendees can look forward to:", ToTwoToo::default());
    }

    #[test]
    fn allow_build_brighter() {
        assert_no_lints(
            "We're empowering them to build brighter futures.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn allow_delegate() {
        assert_no_lints(
            "I’d like you to consciously delegate one task",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_soundscapes() {
        assert_no_lints(
            "Soundscapes are not merely environmental features; they are integral to human identity and cultural expression.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_speed_flexibility() {
        assert_no_lints(
            "Its speed, flexibility, and seamless integration with FZF make it a compelling alternative to traditional fuzzy finding solutions.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_explicitly_cast() {
        assert_no_lints(
            "Attempted to explicitly cast the result back to a string",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_buried_under_data() {
        assert_no_lints(
            "They felt buried under the data, unable to proactively address emerging threats.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_familiarize() {
        assert_no_lints(
            "Familiarize yourself with these resources to learn how to effectively utilize the plugin’s features.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_mapping_commands() {
        assert_no_lints(
            "Mapping Telescope commands to custom keybindings significantly improves workflow.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_great_deal_of_energy() {
        assert_no_lints(
            "It takes a great deal of energy to consistently operate under that kind of pressure.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_occasionally_troubleshoot() {
        assert_no_lints(
            "Just be prepared to occasionally troubleshoot the debugger itself.",
            ToTwoToo::default(),
        );
    }
}
