use crate::linting::LintGroup;

use super::MapPhraseLinterEn;

pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::empty();

    macro_rules! add_compound_mappings {
        ($group:expr, { $($name:expr => ($bad:expr, $good:expr)),+ $(,)? }) => {
            $(
                $group.add(
                    $name,
                    Box::new(MapPhraseLinterEn::new_closed_compound($bad, $good)),
                );
            )+
        };
    }

    // These are compound words that should be condensed.
    // The first column is the name of the rule (which shows up in settings).
    // The second column is the incorrect form of the word and the third column is the correct
    // form.
    add_compound_mappings!(group, {
        "Itself"          => ("it self", "itself"),
        "Myself"          => ("my self", "myself"),
        "Therefore"       => ("there fore", "therefore"),
        "Misunderstand"   => ("miss understand", "misunderstand"),
        "Misunderstood"   => ("miss understood", "misunderstood"),
        "Misuse"          => ("miss use", "misuse"),
        "Misused"         => ("miss used", "misused"),
        "Postpone"        => ("post pone", "postpone"),
        "Worldwide"       => ("world wide", "worldwide"),
        "Overall"         => ("over all", "overall"),
        "However"         => ("how ever", "however"),
        "Upset"           => ("up set", "upset"),
        "Intact"          => ("in tact", "intact"),
        "Somehow"         => ("some how", "somehow"),
        "Proofread"       => ("proof read", "proofread"),
        "Somebody"        => ("some body", "somebody"),
        "Anybody"         => ("any body", "anybody"),
        "Nothing"         => ("no thing", "nothing"),
        "Anywhere"        => ("any where", "anywhere"),
        "Instead"         => ("in stead", "instead"),
        "Somewhere"       => ("some where", "somewhere"),
        "Middleware"      => ("middle ware", "middleware"),
        "Into"            => ("in to", "into"),
        "Overclocking"    => ("over clocking", "overclocking"),
        "Backplane"       => ("back plane", "backplane"),
        "Overload"        => ("over load", "overload"),
        "Underclock"      => ("under clock", "underclock"),
        "Devops"          => ("dev ops", "devops"),
        "Multithreading"  => ("multi threading", "multithreading"),
        "Everywhere"      => ("every where", "everywhere"),
        "Multicore"       => ("multi core", "multicore"),
        "Multimedia"      => ("multi media", "multimedia"),
        "Widespread"      => ("wide spread", "widespread"),
        "Notwithstanding" => ("not with standing", "notwithstanding"),
        "Anyhow"          => ("any how", "anyhow"),
        "Nonetheless"     => ("none the less", "nonetheless"),
        "Thereupon"       => ("there upon", "thereupon"),
        "Insofar"         => ("in so far", "insofar"),
        "Whereupon"       => ("where upon", "whereupon"),
        "Upward"          => ("up ward", "upward"),
        "Henceforth"      => ("hence forth", "henceforth"),
        "Regardless"      => ("regard less", "regardless"),
        "Overnight"       => ("over night", "overnight"),
        "Furthermore"     => ("further more", "furthermore"),
        "Laptop"          => ("lap top", "laptop"),
        "Desktop"         => ("desk top", "desktop"),
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
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn my_self() {
        let test_sentence = "He treated my self with respect.";
        let expected = "He treated myself with respect.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn there_fore() {
        let test_sentence = "This is the reason; there fore, this is true.";
        let expected = "This is the reason; therefore, this is true.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn mis_understood() {
        let test_sentence = "She miss understood the instructions.";
        let expected = "She misunderstood the instructions.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn mis_use() {
        let test_sentence = "He tends to miss use the tool.";
        let expected = "He tends to misuse the tool.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn mis_used() {
        let test_sentence = "The software was miss used.";
        let expected = "The software was misused.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn world_wide() {
        let test_sentence = "The world wide impact was significant.";
        let expected = "The worldwide impact was significant.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn over_all() {
        let test_sentence = "The over all performance was good.";
        let expected = "The overall performance was good.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn how_ever() {
        let test_sentence = "This is true, how ever, details matter.";
        let expected = "This is true, however, details matter.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn wide_spread() {
        let test_sentence = "The news was wide spread throughout the region.";
        let expected = "The news was widespread throughout the region.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn not_with_standing() {
        let test_sentence = "They decided to proceed not with standing any further delay.";
        let expected = "They decided to proceed notwithstanding any further delay.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn any_how() {
        let test_sentence = "She solved the problem any how, even under pressure.";
        let expected = "She solved the problem anyhow, even under pressure.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn none_the_less() {
        let test_sentence = "The results were disappointing, none the less, they continued.";
        let expected = "The results were disappointing, nonetheless, they continued.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn there_upon() {
        let test_sentence = "A decision was made there upon reviewing the data.";
        let expected = "A decision was made thereupon reviewing the data.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn in_so_far() {
        let test_sentence = "This rule applies in so far as it covers all cases.";
        let expected = "This rule applies insofar as it covers all cases.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn where_upon() {
        let test_sentence = "They acted where upon the circumstances allowed.";
        let expected = "They acted whereupon the circumstances allowed.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn up_ward() {
        let test_sentence = "The temperature moved up ward during the afternoon.";
        let expected = "The temperature moved upward during the afternoon.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn hence_forth() {
        let test_sentence = "All new policies apply hence forth immediately.";
        let expected = "All new policies apply henceforth immediately.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn regard_less() {
        let test_sentence = "The decision was made, regard less of the opposition.";
        let expected = "The decision was made, regardless of the opposition.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }

    #[test]
    fn over_night() {
        let test_sentence = "They set off on their journey over night.";
        let expected = "They set off on their journey overnight.";
        assert_suggestion_result(test_sentence, "en", lint_group(), expected);
    }
}
