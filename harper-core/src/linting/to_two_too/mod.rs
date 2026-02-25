mod to_too_adjective_end;
mod to_too_adjective_punct;
mod to_too_adjverb_ed_punct;
mod to_too_adverb;
mod to_too_chunk_start_comma;
mod to_too_degree_words;
mod to_too_eos;
mod to_too_pronoun_end;
mod too_to;

use super::merge_linters::merge_linters;
use super::{ExprLinter, Lint, LintKind, Suggestion};
use to_too_adjective_end::ToTooAdjectiveEnd;
use to_too_adjective_punct::ToTooAdjectivePunct;
use to_too_adjverb_ed_punct::ToTooAdjVerbEdPunct;
use to_too_adverb::ToTooAdverb;
use to_too_chunk_start_comma::ToTooChunkStartComma;
use to_too_degree_words::ToTooDegreeWords;
use to_too_eos::ToTooEos;
use to_too_pronoun_end::ToTooPronounEnd;
use too_to::TooTo;

merge_linters!(
    ToTwoToo =>
        ToTooAdjectiveEnd,
        ToTooAdjectivePunct,
        ToTooAdverb,
        ToTooAdjVerbEdPunct,
        ToTooChunkStartComma,
        ToTooDegreeWords,
        ToTooPronounEnd,
        ToTooEos,
        TooTo
    => "Corrects homophone confusion between `to` and `too`."
);

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    use super::ToTwoToo;

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
    fn no_lint_distance_from_center() {
        assert_no_lints("Distance from the center to any face", ToTwoToo::default());
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

    #[test]
    fn ccoveille_suggestion() {
        assert_no_lints("He goes too far with bets.", ToTwoToo::default());
    }

    #[test]
    fn no_lint_auto_detect_debuggers() {
        assert_no_lints(
            "Daprio attempts to auto-detect debugger servers and configurations, which can save significant time, especially for common languages.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_commitment_open_source() {
        assert_no_lints(
            "I believe a commitment to open-source solutions and internal skill development would ultimately yield a more sustainable and ethical approach.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_feeling_confident_dominate() {
        assert_no_lints(
            "I'm feeling confident, and I suspect you all should be too – because I’m about to dominate.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_egyptian_smiling_faces_commentary() {
        assert_no_lints(
            "Today I learned that the ubiquitous, seemingly cheerful faces carved into ancient Egyptian relief sculptures – often referred to as “smiling faces” – weren’t simply a stylistic choice reflecting happiness. Recent scholarship suggests they functioned as a subtle, often satirical, form of social commentary, particularly targeting individuals who were arrogant, boastful, or otherwise deserving of ridicule.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_intended_to_leave_it_to() {
        assert_no_lints(
            "Beatrice never explicitly said who she intended to leave it to.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_time_for_good_girl_to_bed() {
        assert_no_lints("Time for this good girl to go to bed.", ToTwoToo::default());
    }

    #[test]
    fn no_lint_connected_to_many_other_fields() {
        assert_no_lints(
            "The study is connected to many other fields.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_till_she_too_began_dreaming() {
        assert_no_lints(
            "till she too began dreaming after a fashion",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_to_quickly_find_a_factory() {
        assert_no_lints(
            "To quickly find a factory, look for a map.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_asking_to_simply_in_scare_quotes() {
        assert_no_lints(
            "He was asking to simply \"show up\" for the meeting.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_llm_as_judge_to_automatically_score() {
        assert_no_lints(
            "We used an LLM-as-judge to automatically score agent trajectories.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_all_the_way_to_advanced_usage() {
        assert_no_lints(
            "All the way to advanced usage, like an expert.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_access_to_over_400_integrations() {
        assert_no_lints(
            "You'll have access to over 400 integrations.",
            ToTwoToo::default(),
        );
    }

    #[test]
    fn no_lint_accustomed_to_precision() {
        assert_no_lints("I’m rather accustomed to precision.", ToTwoToo::default());
    }

    #[test]
    fn no_lint_prone_to_melancholy() {
        assert_no_lints("He wasn’t a man prone to melancholy.", ToTwoToo::default());
    }

    #[test]
    fn no_lint_superlative_range() {
        assert_no_lints("Sort speeds from slowest to fastest.", ToTwoToo::default());
    }

    #[test]
    fn no_lint_comparative_range() {
        assert_no_lints("We rank tasks from harder to easier.", ToTwoToo::default());
    }

    #[test]
    fn still_lints_positive_adjective_end() {
        assert_lint_count("The room felt to cold.", ToTwoToo::default(), 1);
    }
}
