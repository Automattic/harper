mod let_us_redundancy;
mod no_contraction_with_verb;

use super::merge_linters::merge_linters;
use let_us_redundancy::LetUsRedundancy;
use no_contraction_with_verb::NoContractionWithVerb;

// See also:
// harper-core/src/linting/compound_nouns/implied_ownership_compound_nouns.rs
// harper-core/src/linting/lets_confusion/let_us_redundancy.rs
// harper-core/src/linting/lets_confusion/no_contraction_with_verb.rs
// harper-core/src/linting/pronoun_contraction/should_contract.rs
merge_linters!(LetsConfusion => LetUsRedundancy, NoContractionWithVerb => "It's often hard to determine where the subject should go with the word `let`. This rule attempts to find common errors with redundancy and contractions that may lead to confusion for readers.");

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::LetsConfusion;

    #[test]
    fn walking() {
        assert_suggestion_result(
            "The crutch let's him walk.",
            LetsConfusion::default(),
            "The crutch lets him walk.",
        );
    }

    #[test]
    fn issue_426_us() {
        assert_suggestion_result("let's us do", LetsConfusion::default(), "lets us do");
    }

    #[test]
    fn issue_426_me() {
        assert_suggestion_result("let's me do", LetsConfusion::default(), "lets me do");
    }

    #[test]
    fn from_harper_docs() {
        assert_suggestion_result(
            "Often the longest and the shortest words are the most helpful, so lets push them first.",
            LetsConfusion::default(),
            "Often the longest and the shortest words are the most helpful, so let's push them first.",
        );
    }

    // "play" is also a noun so in a context like "Sometimes the umpire lets play continue"
    // #[test]
    // fn issue_470_missing_apostrophe() {
    //     assert_suggestion_result("lets play", LetsConfusion::default(), "let's play");
    // }

    // #[test]
    // fn issue_470_missing_subject() {
    //     assert_suggestion_result("let play", LetsConfusion::default(), "let's play");
    // }

    #[test]
    fn issue_470_missing_apostrophe() {
        assert_suggestion_result("lets proceed", LetsConfusion::default(), "let's proceed");
    }

    #[test]
    fn issue_470_missing_subject() {
        assert_suggestion_result("let proceed", LetsConfusion::default(), "let's proceed");
    }

    #[test]
    fn issue_548() {
        assert_lint_count(
            "A simple web app that lets you fetch random issues.",
            LetsConfusion::default(),
            0,
        );
    }
}
