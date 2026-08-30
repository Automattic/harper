use super::merge_linters::merge_linters;

mod to_wary;
mod to_weary;
use to_wary::ToWary;
use to_weary::ToWeary;

merge_linters!(WaryWeary => ToWary, ToWeary => "Handles common confusions between `wary` (cautious) and `weary` (tired), such as `weary eye` (should be `wary eye`) and `bone wary` (should be `bone weary`).");

#[cfg(test)]
mod tests {
    use super::WaryWeary;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn corrects_weary_eye_to_wary() {
        assert_suggestion_result(
            "She kept a weary eye on the exits.",
            WaryWeary::default(),
            "She kept a wary eye on the exits.",
        );
    }

    #[test]
    fn corrects_weary_eyes_to_wary() {
        assert_suggestion_result(
            "They watched with weary eyes for any danger.",
            WaryWeary::default(),
            "They watched with wary eyes for any danger.",
        );
    }

    #[test]
    fn corrects_mixed_case_weary_eye() {
        assert_suggestion_result(
            "He cast a Weary eye over the crowd.",
            WaryWeary::default(),
            "He cast a Wary eye over the crowd.",
        );
    }

    #[test]
    fn corrects_bone_wary_to_weary() {
        assert_suggestion_result(
            "After the march they were bone wary.",
            WaryWeary::default(),
            "After the march they were bone weary.",
        );
    }

    #[test]
    fn corrects_world_wary_to_weary() {
        assert_suggestion_result(
            "The old sailor had a world wary look.",
            WaryWeary::default(),
            "The old sailor had a world weary look.",
        );
    }

    #[test]
    fn does_not_touch_correct_wary_eye() {
        assert_suggestion_result(
            "She kept a wary eye on the exits.",
            WaryWeary::default(),
            "She kept a wary eye on the exits.",
        );
    }

    #[test]
    fn does_not_touch_correct_bone_weary() {
        assert_suggestion_result(
            "After the march they were bone weary.",
            WaryWeary::default(),
            "After the march they were bone weary.",
        );
    }

    #[test]
    fn does_not_touch_ambiguous_weary_of() {
        // "weary of" (tired of) is valid and must not be flagged.
        assert_suggestion_result(
            "I am weary of these long meetings.",
            WaryWeary::default(),
            "I am weary of these long meetings.",
        );
    }
}
