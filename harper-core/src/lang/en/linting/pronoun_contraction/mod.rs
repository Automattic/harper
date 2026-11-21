use super::merge_linters::merge_linters;

mod avoid_contraction;
mod should_contract;

use avoid_contraction::AvoidContraction;
use should_contract::ShouldContract;

merge_linters! {PronounContraction => ShouldContract, AvoidContraction => "Choosing when to contract pronouns is a challenging art. This rule looks for faults." }

#[cfg(test)]
mod tests {
    use super::PronounContraction;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn issue_225() {
        assert_suggestion_result(
            "Your the man",
            PronounContraction::default(),
            "You're the man",
        );
    }

    #[test]
    fn were_team() {
        assert_suggestion_result(
            "Were the best team.",
            PronounContraction::default(),
            "We're the best team.",
        );
    }

    #[test]
    fn issue_139() {
        assert_suggestion_result(
            "it would be great if you're PR was merged into tower-lsp",
            PronounContraction::default(),
            "it would be great if your PR was merged into tower-lsp",
        );
    }

    #[test]
    fn car() {
        assert_suggestion_result(
            "You're car is black.",
            PronounContraction::default(),
            "Your car is black.",
        );
    }

    #[test]
    fn allows_you_are_still() {
        assert_lint_count(
            "In case you're still not convinced.",
            PronounContraction::default(),
            0,
        );
    }

    #[test]
    fn issue_576() {
        assert_lint_count(
            "If you're not happy you try again.",
            PronounContraction::default(),
            0,
        );
        assert_lint_count("No you're not.", PronounContraction::default(), 0);
        assert_lint_count(
            "Even if you're not fluent in arm assembly, you surely noticed this.",
            PronounContraction::default(),
            0,
        );
    }
}
