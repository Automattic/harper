use crate::patterns::{ExactPhrase, OwnedPatternExt};

use super::{LintGroup, MapPhraseLinter};

/// Produce a [`LintGroup`] that looks for errors in common phrases.
/// Comes pre-configured with the recommended default settings.
pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::default();

    macro_rules! add_similar_mappings {
        ($group:expr, { $($name:expr => ($phrase:expr, $threshold:expr)),+ $(,)? }) => {
            $(
                $group.add(
                    $name,
                    Box::new(MapPhraseLinter::new_similar_to_phrase($phrase, $threshold)),
                );
            )+
        };
    }

    // These are rules that look for phrases that are _almost_ but not exactly
    // equal to the correct form (shown on the right).
    //
    // The number is the allowed edit distance per word to search for.
    add_similar_mappings!(group, {
        "TurnItOff"          => ("turn it off", 1),
        "HumanLife"          => ("human life", 1),
        "ThatChallenged"     => ("that challenged", 2),
        "NoLonger"           => ("no longer", 1),
        "NeedHelp"           => ("need help", 1),
        "OfCourse"           => ("of course", 1),
        "AndTheLike"         => ("and the like", 1),
        "BadRap"             => ("bad rap", 1),
        "BatedBreath"        => ("bated breath", 1),
        "BeckAndCall"        => ("beck and call", 1),
        "HungerPang"         => ("hunger pang", 2),
        "EnMasse"            => ("en masse", 1),
        "LetAlone"           => ("let alone", 1),
        "SneakingSuspicion"  => ("sneaking suspicion", 3),
        "SpecialAttention"   => ("special attention", 1),
        "SupposedTo"         => ("supposed to", 1),
        "KindRegards"         => ("kind regards", 1),
        "ThoughtProcess"         => ("thought process", 1),
    });

    macro_rules! add_exact_mappings {
        ($group:expr, {
            $($name:expr => ($input:expr, $corrections:expr, $hint:expr, $description:expr)),+ $(,)?
        }) => {
            $(
                $group.add(
                    $name,
                    Box::new(MapPhraseLinter::new_exact_phrase(
                        $input,
                        $corrections,
                        $hint,
                        $description,
                    )),
                );
            )+
        };
    }

    add_exact_mappings!(group, {
        // The name of the rule
        "ChangeTack" => (
            // The exact phrase to look for.
            "change tact",
            // The corrections to provide.
            ["change tack"],
            // The message to be shown with the error.
            "Did you mean the sailing idiom?",
            // A description of the rule.
            "Locates minor errors in the sailing idiom `change tack`."
        ),
        "WantBe" => (
            "want be",
            ["won't be", "want to be"],
            "Did you mean `won't be` or `want to be`?",
            "Detects incorrect usage of `want be` and suggests `won't be` or `want to be` based on context."
        ),
        "StateOfTheArt" => (
            "state of art",
            ["state of the art"],
            "Did you mean `state of the art`?",
            "Detects incorrect usage of `state of art` and suggests `state of the art` as the correct phrase."
        ),
        "FaceFirst" => (
            "face first into",
            ["face-first into"],
            "Should this be `face-first`?",
            "Ensures `face first` is correctly hyphenated as `face-first` when used before `into`."
        ),
        "EludedTo" => (
            "eluded to",
            ["alluded to"],
            "Did you mean `alluded to`?",
            "Corrects `eluded to` to `alluded to` in contexts referring to indirect references."
        ),
        "BaitedBreath" => (
            "baited breath",
            ["bated breath"],
            "Did you mean `bated breath`?",
            "Ensures `bated breath` is written correctly, as `baited breath` is incorrect."
        ),
        "BareInMind" => (
            "bare in mind",
            ["bear in mind"],
            "Did you mean `bear in mind`?",
            "Ensures the phrase `bear in mind` is used correctly instead of `bare in mind`."
        ),
        "MutePoint" => (
            "mute point",
            ["moot point"],
            "Did you mean `moot point`?",
            "Ensures `moot point` is used instead of `mute point`, as `moot` means debatable or irrelevant."
        ),
        "RoadMap" => (
            "roadmap",
            ["road map"],
            "Did you mean `road map`?",
            "Detects when `roadmap` is used instead of `road map`, prompting the correct spacing."
        ),
        "SameAs" => (
            "same then",
            ["same as"],
            "Did you mean `same as`?",
            "Corrects the incorrect phrase `same then` to the standard `same as`."
        ),
        "SoonerOrLater" => (
            "sooner than later",
            ["sooner rather than later", "sooner or later"],
            "Did you mean `sooner rather than later` or `sooner or later`?",
            "Fixes the improper phrase `sooner than later` by suggesting standard alternatives."
        ),
        "HadOf" => (
            "had of",
            ["had have", "had've"],
            "Did you mean `had have` or `had've`?",
            "Flags the unnecessary use of `of` after `had` and suggests the correct forms."
        ),
        "FatalOutcome" => (
            "fatal outcome",
            ["death"],
            "Consider using `death` for clarity.",
            "Replaces `fatal outcome` with the more direct term `death` for conciseness."
        ),
        "NotTo" => (
            "no to",
            ["not to"],
            "Did you mean `not to`?",
            "Corrects `no to` to `not to`, ensuring proper negation."
        ),
        "ThatThis" => (
            "the this",
            ["that this"],
            "Did you mean `that this`?",
            "Fixes `the this` to the correct phrase `that this`."
        ),
        "CondenseAllThe" => (
            "all of the",
            ["all the"],
            "Consider simplifying to `all the`.",
            "Suggests removing `of` in `all of the` for a more concise phrase."
        ),
        "AvoidAndAlso" => (
            "and also",
            ["and"],
            "Consider using just `and`.",
            "Reduces redundancy by replacing `and also` with `and`."
        ),
        "AndIn" => (
            "an in",
            ["and in"],
            "Did you mean `and in`?",
            "Fixes the incorrect phrase `an in` to `and in` for proper conjunction usage."
        ),
        "BeenThere" => (
            "bee there",
            ["been there"],
            "Did you mean `been there`?",
            "Corrects the misspelling `bee there` to the proper phrase `been there`."
        ),
        "CanBeSeen" => (
            "can be seem",
            ["can be seen"],
            "Did you mean `can be seen`?",
            "Corrects `can be seem` to the proper phrase `can be seen`."
        ),
        "GoingTo" => (
            "gong to",
            ["going to"],
            "Did you mean `going to`?",
            "Corrects `gong to` to the intended phrase `going to`."
        ),
        "IAm" => (
            "I a m",
            ["I am"],
            "Did you mean `I am`?",
            "Fixes the incorrect spacing in `I a m` to properly form `I am`."
        ),
        "ItCan" => (
            "It cam",
            ["It can"],
            "Did you mean `It can`?",
            "Corrects the misspelling `It cam` to the proper phrase `It can`."
        ),
        "MyHouse" => (
            "mu house",
            ["my house"],
            "Did you mean `my house`?",
            "Fixes the typo `mu house` to `my house`."
        ),
        "FarToMany" => (
            "far to many",
            ["far too many"],
            "Did you mean `far too many`?",
            "Ensures `too many` is used instead of `to many`."
        ),
        "FarToMuch" => (
            "far to much",
            ["far too much"],
            "Did you mean `far too much`?",
            "Ensures `too much` is used instead of `to much`."
        ),
        "WayToMany" => (
            "way to many",
            ["way too many"],
            "Did you mean `way too many`?",
            "Ensures `way too many` is used instead of `to many`."
        ),
        "WayToMuch" => (
            "way to much",
            ["way too much"],
            "Did you mean `way too much`?",
            "Ensures `way too much` is used instead of `to much`."
        ),
        "BanTogether" => (
            "ban together",
            ["band together"],
            "Did you mean 'band together'?",
            "Detects and corrects the common error of using `ban together` instead of the idiom `band together`, which means to unite or join forces."
        )
    });

    group.add("FastPaste", Box::new(MapPhraseLinter::new(
            Box::new(ExactPhrase::from_phrase("fast paste").or(Box::new(ExactPhrase::from_phrase("fast-paste")))),
            ["fast-paced"],
            "Did you mean `fast-paced`?",
            "Detects incorrect usage of `fast paste` or `fast-paste` and suggests `fast-paced` as the correct phrase."
        )));

    group.set_all_rules_to(Some(true));

    group
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::lint_group;

    #[test]
    fn issue_574() {
        assert_lint_count("run by one", lint_group(), 0);
    }

    #[test]
    fn turn_it_off_clean_lower() {
        assert_lint_count("turn it off", lint_group(), 0);
    }

    #[test]
    fn turn_it_off_clean_upper() {
        assert_lint_count("Turn it off", lint_group(), 0);
    }

    #[test]
    fn of_confusion() {
        assert_suggestion_result("Turn it of", lint_group(), "Turn it off");
    }

    #[test]
    fn i_and_of_confusion() {
        assert_suggestion_result("Turn i of", lint_group(), "Turn it off");
    }

    #[test]
    fn off_course() {
        assert_suggestion_result(
            "Yes, off course we should do that.",
            lint_group(),
            "Yes, of course we should do that.",
        );
    }

    #[test]
    fn o_course() {
        assert_suggestion_result(
            "Yes, o course we should do that.",
            lint_group(),
            "Yes, of course we should do that.",
        );
    }

    #[test]
    fn bad_rep() {
        assert_suggestion_result("bad rep", lint_group(), "bad rap");
    }

    #[test]
    fn baited_breath() {
        assert_suggestion_result("baited breath", lint_group(), "bated breath");
    }

    #[test]
    fn change_tact() {
        assert_suggestion_result("change tact", lint_group(), "change tack");
    }

    #[test]
    fn hunger_pain() {
        assert_suggestion_result("hunger pain", lint_group(), "hunger pang");
    }

    #[test]
    fn in_mass() {
        assert_suggestion_result("in mass", lint_group(), "en masse");
    }

    #[test]
    fn let_along() {
        assert_suggestion_result("let along", lint_group(), "let alone");
    }

    #[test]
    fn sneaky_suspicion() {
        assert_suggestion_result("sneaky suspicion", lint_group(), "sneaking suspicion");
    }

    #[test]
    fn supposed_to() {
        assert_suggestion_result("suppose to", lint_group(), "supposed to");
    }

    #[test]
    fn spacial_attention() {
        assert_suggestion_result("spacial attention", lint_group(), "special attention");
    }

    #[test]
    fn now_on_hold() {
        assert_lint_count("Those are now on hold for month.", lint_group(), 0);
    }

    #[test]
    fn far_to_many() {
        assert_suggestion_result(
            "darknet detecting far to many objects and in random locations",
            lint_group(),
            "darknet detecting far too many objects and in random locations",
        );
    }

    fn far_to_much() {
        assert_suggestion_result(
            "requires far to much day today support and troubleshooting",
            lint_group(),
            "requires far too much day today support and troubleshooting",
        );
    }

    fn way_to_many() {
        assert_suggestion_result(
            "Way to many TEMP files",
            lint_group(),
            "way too many TEMP files",
        );
    }

    fn way_to_much() {
        assert_suggestion_result(
            "proper java development has way to much overhead",
            lint_group(),
            "proper java development has way too much overhead",
        );
    }
}
