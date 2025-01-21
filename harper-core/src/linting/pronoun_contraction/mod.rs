use super::merge_linters::merge_linters;

mod avoid_contraction;
mod should_contract;

use avoid_contraction::AvoidContraction;
use should_contract::ShouldContract;

merge_linters! {PronounContraction => ShouldContract, AvoidContraction => "Choosing when to contract pronouns is a challenging art. This rule looks for faults." }

#[cfg(test)]
mod tests {
    use super::PronounContraction;
    use crate::linting::tests::assert_suggestion_result;

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
}
