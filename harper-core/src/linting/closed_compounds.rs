use crate::linting::LintGroup;

use super::MapPhraseLinter;

pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::empty();

    macro_rules! add_compound_mappings {
        ($group:expr, { $($name:expr => ($bad:expr, $good:expr)),+ $(,)? }) => {
            $(
                $group.add(
                    $name,
                    Box::new(MapPhraseLinter::new_closed_compound($bad, $good)),
                );
            )+
        };
    }

    // These are compound words that should be condensed.
    // The first column is the name of the rule (which shows up in settings).
    // The second column is the incorrect form of the word and the third column is the correct
    // form.
    add_compound_mappings!(group, {
        "Anybody"         => ("any body", "anybody"),
        "Anyhow"          => ("any how", "anyhow"),
        "Anywhere"        => ("any where", "anywhere"),
        "Backplane"       => ("back plane", "backplane"),
        "Bypass"          => ("by pass", "bypass"),
        "Deadlift"        => ("dead lift", "deadlift"),
        "Desktop"         => ("desk top", "desktop"),
        "Devops"          => ("dev ops", "devops"),
        "Everybody"       => ("every body", "everybody"),
        "Everyone"        => ("every one", "everyone"),
        "Everywhere"      => ("every where", "everywhere"),
        "Furthermore"     => ("further more", "furthermore"),
        "Henceforth"      => ("hence forth", "henceforth"),
        "However"         => ("how ever", "however"),
        "Insofar"         => ("in so far", "insofar"),
        "Instead"         => ("in stead", "instead"),
        "Intact"          => ("in tact", "intact"),
        "Itself"          => ("it self", "itself"),
        "Keystroke"       => ("key stoke", "keystroke"),
        "Keystrokes"      => ("key stokes", "keystrokes"),
        "Laptop"          => ("lap top", "laptop"),
        "Middleware"      => ("middle ware", "middleware"),
        "Misunderstand"   => ("miss understand", "misunderstand"),
        "Misunderstood"   => ("miss understood", "misunderstood"),
        "Misuse"          => ("miss use", "misuse"),
        "Misused"         => ("miss used", "misused"),
        "Multicore"       => ("multi core", "multicore"),
        "Multimedia"      => ("multi media", "multimedia"),
        "Multithreading"  => ("multi threading", "multithreading"),
        "Myself"          => ("my self", "myself"),
        "Nonetheless"     => ("none the less", "nonetheless"),
        "Nobody"          => ("no body", "nobody"),
        "Nowhere"         => ("no where", "nowhere"),
        "Nothing"         => ("no thing", "nothing"),
        "Notwithstanding" => ("not with standing", "notwithstanding"),
        "Overall"         => ("over all", "overall"),
        "Overclocking"    => ("over clocking", "overclocking"),
        "Overload"        => ("over load", "overload"),
        "Overnight"       => ("over night", "overnight"),
        "Postpone"        => ("post pone", "postpone"),
        "Proofread"       => ("proof read", "proofread"),
        "Regardless"      => ("regard less", "regardless"),
        "Shortcoming"     => ("short coming", "shortcoming"),
        "Shortcomings"    => ("short comings", "shortcomings"),
        "Somebody"        => ("some body", "somebody"),
        "Somehow"         => ("some how", "somehow"),
        "Someone"         => ("some one", "someone"),
        "Somewhere"       => ("some where", "somewhere"),
        "There"           => ("the re", "there"),
        "Therefore"       => ("there fore", "therefore"),
        "Thereupon"       => ("there upon", "thereupon"),
        "Underclock"      => ("under clock", "underclock"),
        "Upset"           => ("up set", "upset"),
        "Upward"          => ("up ward", "upward"),
        "Whereupon"       => ("where upon", "whereupon"),
        "Widespread"      => ("wide spread", "widespread"),
        "Without"         => ("with out", "without"),
        "Worldwide"       => ("world wide", "worldwide"),
    });

    group.set_all_rules_to(Some(true));

    group
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

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
    fn key_stoke() {
        let test_sentence = "Use this key stoke to open search.";
        let expected = "Use this keystroke to open search.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn key_stokes() {
        let test_sentence = "These key stokes are hard to memorize.";
        let expected = "These keystrokes are hard to memorize.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
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
}
