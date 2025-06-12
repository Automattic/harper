use super::merge_linters::merge_linters;

mod to_hop;
mod to_hope;
use to_hop::ToHop;
use to_hope::ToHope;

merge_linters!(HopHope => ToHop, ToHope => "Handles common errors involving `hop` and `hope`. Ensures `hop` is used correctly in phrases like `hop on a bus` while correcting mistaken uses of `hope` in contexts where `hop` is expected.");

#[cfg(test)]
mod tests {
    use super::HopHope;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn corrects_hop_to_hope() {
        assert_suggestion_result(
            "I hop we can clarify this soon.",
            HopHope::default(),
            "I hope we can clarify this soon.",
        );
    }

    #[test]
    fn does_not_correct_unrelated_use() {
        assert_suggestion_result(
            "I hop on one foot for fun.",
            HopHope::default(),
            "I hop on one foot for fun.",
        );
    }

    #[test]
    fn corrects_mixed_case_hop() {
        assert_suggestion_result(
            "I HoP we can find a solution.",
            HopHope::default(),
            "I HoPE we can find a solution.",
        );
    }

    #[test]
    fn corrects_hoping_on_call() {
        assert_suggestion_result(
            "I was hoping on a call to discuss this.",
            HopHope::default(),
            "I was hopping on a call to discuss this.",
        );
    }

    #[test]
    fn corrects_hoped_on_plane() {
        assert_suggestion_result(
            "She hoped on an airplane to visit family.",
            HopHope::default(),
            "She hopped on an airplane to visit family.",
        );
    }

    #[test]
    fn corrects_hope_on_bus() {
        assert_suggestion_result(
            "They hope on a bus every morning.",
            HopHope::default(),
            "They hop on a bus every morning.",
        );
    }

    #[test]
    fn does_not_correct_unrelated_context() {
        assert_suggestion_result(
            "I hope everything goes well with your project.",
            HopHope::default(),
            "I hope everything goes well with your project.",
        );
    }

    #[test]
    fn corrects_mixed_case() {
        assert_suggestion_result(
            "She HoPeD on a train to get home.",
            HopHope::default(),
            "She HoPpED on a train to get home.",
        );
    }
}
