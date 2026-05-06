use crate::linting::LintGroup;

use super::MapPhraseLinter;

pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::empty();

    macro_rules! add_compound_mappings {
        ($group:expr, { $($parts:expr => $compound:expr),+ $(,)? }) => {
            $(
                let name = {
                    let mut chars = $compound.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                };
                $group.add(
                    &name,
                    Box::new(MapPhraseLinter::new_closed_compound($parts, $compound)),
                );
            )+
        };
    }

    // These are compound words that should be condensed.
    // The first column is the name of the rule (which shows up in settings).
    // The second column is the incorrect form of the word and the third column is the correct
    // form.
    add_compound_mappings!(group, {
        &["any", "body"][..] => "anybody",
        &["any", "how"][..] => "anyhow",
        &["any", "where"][..] => "anywhere",
        &["back", "plane"][..] => "backplane",
        &["by", "pass"][..] => "bypass",
        &["chalk", "board"][..] => "chalkboard",
        &["dead", "lift"][..] => "deadlift",
        &["desk", "top"][..] => "desktop",
        &["dev", "ops"][..] => "devops",
        &["every", "body"][..] => "everybody",
        &["every", "one"][..] => "everyone",
        &["every", "where"][..] => "everywhere",
        &["further", "more"][..] => "furthermore",
        &["hence", "forth"][..] => "henceforth",
        &["how", "ever"][..] => "however",
        &["in", "so", "far"][..] => "insofar",
        &["in", "stead"][..] => "instead",
        &["in", "tact"][..] => "intact",
        &["it", "self"][..] => "itself",
        &["lap", "top"][..] => "laptop",
        &["middle", "ware"][..] => "middleware",
        &["mean", "while"][..] => "meanwhile",
        &["miss", "understand"][..] => "misunderstand",
        &["miss", "understood"][..] => "misunderstood",
        &["miss", "use"][..] => "misuse",
        &["miss", "used"][..] => "misused",
        &["multi", "core"][..] => "multicore",
        &["multi", "media"][..] => "multimedia",
        &["multi", "threading"][..] => "multithreading",
        &["my", "self"][..] => "myself",
        &["none", "the", "less"][..] => "nonetheless",
        &["no", "thing"][..] => "nothing",
        &["not", "with", "standing"][..] => "notwithstanding",
        &["no", "where"][..] => "nowhere",
        &["over", "all"][..] => "overall",
        &["over", "clocking"][..] => "overclocking",
        &["over", "load"][..] => "overload",
        &["over", "night"][..] => "overnight",
        &["post", "pone"][..] => "postpone",
        &["proof", "read"][..] => "proofread",
        &["regard", "less"][..] => "regardless",
        &["short", "coming"][..] => "shortcoming",
        &["short", "comings"][..] => "shortcomings",
        &["some", "body"][..] => "somebody",
        &["some", "how"][..] => "somehow",
        &["some", "one"][..] => "someone",
        &["some", "where"][..] => "somewhere",
        &["the", "re"][..] => "there",
        &["there", "fore"][..] => "therefore",
        &["there", "upon"][..] => "thereupon",
        &["under", "clock"][..] => "underclock",
        &["up", "set"][..] => "upset",
        &["up", "ward"][..] => "upward",
        &["where", "upon"][..] => "whereupon",
        &["wide", "spread"][..] => "widespread",
        &["with", "out"][..] => "without",
        &["world", "wide"][..] => "worldwide",
        &["worth", "while"][..] => "worthwhile",
    });

    group.set_all_rules_to(Some(true));

    group
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::lint_group;

    #[test]
    fn it_self() {
        let test_sentence = "The project, it self, was quite challenging.";
        let expected = "The project, itself, was quite challenging.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn my_self() {
        let test_sentence = "He treated my self with respect.";
        let expected = "He treated myself with respect.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn there_fore() {
        let test_sentence = "This is the reason; there fore, this is true.";
        let expected = "This is the reason; therefore, this is true.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn mis_understood() {
        let test_sentence = "She miss understood the instructions.";
        let expected = "She misunderstood the instructions.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn mis_use() {
        let test_sentence = "He tends to miss use the tool.";
        let expected = "He tends to misuse the tool.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn mis_used() {
        let test_sentence = "The software was miss used.";
        let expected = "The software was misused.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn mean_while() {
        let test_sentence = "Mean while, the team kept working.";
        let expected = "Meanwhile, the team kept working.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn world_wide() {
        let test_sentence = "The world wide impact was significant.";
        let expected = "The worldwide impact was significant.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn over_all() {
        let test_sentence = "The over all performance was good.";
        let expected = "The overall performance was good.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn how_ever() {
        let test_sentence = "This is true, how ever, details matter.";
        let expected = "This is true, however, details matter.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn wide_spread() {
        let test_sentence = "The news was wide spread throughout the region.";
        let expected = "The news was widespread throughout the region.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn not_with_standing() {
        let test_sentence = "They decided to proceed not with standing any further delay.";
        let expected = "They decided to proceed notwithstanding any further delay.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn any_how() {
        let test_sentence = "She solved the problem any how, even under pressure.";
        let expected = "She solved the problem anyhow, even under pressure.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn none_the_less() {
        let test_sentence = "The results were disappointing, none the less, they continued.";
        let expected = "The results were disappointing, nonetheless, they continued.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn there_upon() {
        let test_sentence = "A decision was made there upon reviewing the data.";
        let expected = "A decision was made thereupon reviewing the data.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn in_so_far() {
        let test_sentence = "This rule applies in so far as it covers all cases.";
        let expected = "This rule applies insofar as it covers all cases.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn where_upon() {
        let test_sentence = "They acted where upon the circumstances allowed.";
        let expected = "They acted whereupon the circumstances allowed.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn up_ward() {
        let test_sentence = "The temperature moved up ward during the afternoon.";
        let expected = "The temperature moved upward during the afternoon.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn hence_forth() {
        let test_sentence = "All new policies apply hence forth immediately.";
        let expected = "All new policies apply henceforth immediately.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn regard_less() {
        let test_sentence = "The decision was made, regard less of the opposition.";
        let expected = "The decision was made, regardless of the opposition.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn over_night() {
        let test_sentence = "They set off on their journey over night.";
        let expected = "They set off on their journey overnight.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn by_pass() {
        let test_sentence = "Please by pass this check for now.";
        let expected = "Please bypass this check for now.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn dead_lift() {
        let test_sentence = "I can dead lift 200 kg.";
        let expected = "I can deadlift 200 kg.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn chalk_board() {
        let test_sentence = "The teacher wrote the equation on the chalk board.";
        let expected = "The teacher wrote the equation on the chalkboard.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn in_tact_space() {
        let test_sentence = "The code remains in tact after the merge.";
        let expected = "The code remains intact after the merge.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn in_tact_hyphen() {
        let test_sentence = "The code remains in-tact after the merge.";
        let expected = "The code remains intact after the merge.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn intact_is_allowed() {
        assert_no_lints("The data set remains intact.", lint_group());
    }

    #[test]
    fn with_out() {
        let test_sentence = "We left with out a map.";
        let expected = "We left without a map.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn the_re() {
        let test_sentence = "The re are too many popups on this page.";
        let expected = "There are too many popups on this page.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn short_coming() {
        let test_sentence = "That bug is a short coming in the current release.";
        let expected = "That bug is a shortcoming in the current release.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn short_comings() {
        let test_sentence = "We listed three short comings in the postmortem.";
        let expected = "We listed three shortcomings in the postmortem.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn worth_while() {
        let test_sentence =
            "It's worth while documenting all the clientside events that the eventsService emits?";
        let expected =
            "It's worthwhile documenting all the clientside events that the eventsService emits?";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn worth_hyphen_while() {
        let test_sentence = "I feel that the special case of looping over sequences that follow a standard iterator protocol (i.e. optionals) is important enough to be worth-while.";
        let expected = "I feel that the special case of looping over sequences that follow a standard iterator protocol (i.e. optionals) is important enough to be worthwhile.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }
}
