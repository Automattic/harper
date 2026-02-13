use super::merge_linters::merge_linters;

mod general;
mod proper_noun;

use general::General;
use proper_noun::ProperNoun;

merge_linters!(
    ItsContraction => General, ProperNoun =>
    "Detects places where the possessive `its` should be the contraction `it's`, including before verbs/clauses and before proper nouns after opinion verbs."
);

#[cfg(test)]
mod tests {
    use super::ItsContraction;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn fix_had() {
        assert_suggestion_result(
            "Its had an enormous effect.",
            ItsContraction::default(),
            "It's had an enormous effect.",
        );
    }

    #[test]
    fn fix_been() {
        assert_suggestion_result(
            "Its been months since we spoke.",
            ItsContraction::default(),
            "It's been months since we spoke.",
        );
    }

    #[test]
    fn fix_got() {
        assert_suggestion_result(
            "I think its got nothing to do with us.",
            ItsContraction::default(),
            "I think it's got nothing to do with us.",
        );
    }

    #[test]
    fn fixes_its_common() {
        assert_suggestion_result(
            "Its common for users to get frustrated.",
            ItsContraction::default(),
            "It's common for users to get frustrated.",
        );
    }

    #[test]
    fn ignore_correct_contraction() {
        assert_lint_count(
            "It's been a long year for everyone.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn ignore_possessive() {
        assert_lint_count(
            "The company revised its policies last week.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn ignore_coroutine() {
        assert_lint_count(
            "Launch each task within its own child coroutine.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn issue_381() {
        assert_suggestion_result(
            "Its a nice day.",
            ItsContraction::default(),
            "It's a nice day.",
        );
    }

    #[test]
    fn ignore_nominal_progressive() {
        assert_lint_count(
            "The class preserves its existing properties.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    #[ignore = "past participles are not always adjectives ('cared' for instance)"]
    fn ignore_nominal_perfect() {
        assert_lint_count(
            "The robot followed its predetermined route.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn ignore_nominal_long() {
        assert_lint_count(
            "I think of its exploding marvelous spectacular output.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn corrects_because() {
        assert_suggestion_result(
            "Its because they don't want to.",
            ItsContraction::default(),
            "It's because they don't want to.",
        );
    }

    #[test]
    fn corrects_its_hard() {
        assert_suggestion_result(
            "Its hard to believe that.",
            ItsContraction::default(),
            "It's hard to believe that.",
        );
    }

    #[test]
    fn corrects_its_easy() {
        assert_suggestion_result(
            "Its easy if you try.",
            ItsContraction::default(),
            "It's easy if you try.",
        );
    }

    #[test]
    fn corrects_its_a_picnic() {
        assert_suggestion_result(
            "Its a beautiful day for a picnic",
            ItsContraction::default(),
            "It's a beautiful day for a picnic",
        );
    }

    #[test]
    fn corrects_its_my() {
        assert_suggestion_result(
            "Its my favorite song.",
            ItsContraction::default(),
            "It's my favorite song.",
        );
    }

    #[test]
    fn allows_its_new() {
        assert_no_lints(
            "The company announced its new product line. ",
            ItsContraction::default(),
        );
    }

    #[test]
    fn allows_its_own_charm() {
        assert_no_lints("The house has its own charm. ", ItsContraction::default());
    }

    #[test]
    fn allows_its_victory() {
        assert_no_lints(
            "The team celebrated its victory. ",
            ItsContraction::default(),
        );
    }

    #[test]
    fn allows_its_history() {
        assert_no_lints(
            "The country is proud of its history. ",
            ItsContraction::default(),
        );
    }

    #[test]
    fn allows_its_secrets() {
        assert_no_lints(
            "The book contains its own secrets. ",
            ItsContraction::default(),
        );
    }

    #[test]
    fn corrects_think_google() {
        assert_suggestion_result(
            "I think its Google, not Microsoft.",
            ItsContraction::default(),
            "I think it's Google, not Microsoft.",
        );
    }

    #[test]
    fn corrects_hope_katie() {
        assert_suggestion_result(
            "I hope its Katie.",
            ItsContraction::default(),
            "I hope it's Katie.",
        );
    }

    #[test]
    fn corrects_guess_date() {
        assert_suggestion_result(
            "I guess its March 6.",
            ItsContraction::default(),
            "I guess it's March 6.",
        );
    }

    #[test]
    fn corrects_assume_john() {
        assert_suggestion_result(
            "We assume its John.",
            ItsContraction::default(),
            "We assume it's John.",
        );
    }

    #[test]
    fn corrects_doubt_tesla() {
        assert_suggestion_result(
            "They doubt its Tesla this year.",
            ItsContraction::default(),
            "They doubt it's Tesla this year.",
        );
    }

    #[test]
    fn handles_two_word_name() {
        assert_suggestion_result(
            "She thinks its New York.",
            ItsContraction::default(),
            "She thinks it's New York.",
        );
    }

    #[test]
    fn ignores_existing_contraction() {
        assert_lint_count("I think it's Google.", ItsContraction::default(), 0);
    }

    #[test]
    fn ignores_possessive_noun_after_name() {
        assert_lint_count(
            "I think its Google product launch.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn ignores_without_opinion_verb() {
        assert_lint_count(
            "Its Google Pixel lineup is impressive.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn ignores_common_noun_target() {
        assert_lint_count(
            "We hope its accuracy improves.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn issue_2547() {
        assert_no_lints(
            "using the foo feature and its associated parameter",
            ItsContraction::default(),
        );
    }

    #[test]
    fn ignore_past_participle_noun_phrase() {
        assert_no_lints(
            "using the foo feature and its abetted parameter",
            ItsContraction::default(),
        );
    }

    #[test]
    fn corrects_predicative_called() {
        assert_suggestion_result(
            "Its called recursion.",
            ItsContraction::default(),
            "It's called recursion.",
        );
    }

    #[test]
    fn corrects_predicative_named() {
        assert_suggestion_result(
            "Its named Manhattan.",
            ItsContraction::default(),
            "It's named Manhattan.",
        );
    }

    #[test]
    fn allows_possessive_generated_code() {
        assert_no_lints(
            "The compiler emits its generated code.",
            ItsContraction::default(),
        );
    }
}
