use crate::linting::{LintGroup, LintKind};

use super::MapPhraseLinter;

pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::empty();

    macro_rules! add_compound_mappings {
        ($group:expr, { $($name:expr => $mappings:expr),+ $(,)? }) => {
            $(
                let mut all_patterns: Vec<&[&str]> = Vec::new();
                let mut all_correct: Vec<&str> = Vec::new();

                for (bads, good) in $mappings {
                    all_patterns.push(*bads);
                    all_correct.push(*good);
                }

                let patterns_flat: Vec<&str> = all_patterns.iter().flat_map(|p| p.iter().copied()).collect();
                let first_correct = if let Some(first) = all_correct.first() {
                    *first
                } else {
                    ""
                };

                $group.add(
                    $name,
                    Box::new(MapPhraseLinter::new_fixed_phrases(
                        patterns_flat,
                        all_correct,
                        format!("Did you mean the closed compound `{}`?", first_correct),
                        format!("Looks for incorrect spacing inside closed compounds."),
                        Some(LintKind::Miscellaneous),
                    )),
                );
            )+
        };
    }

    // These are compound words that should be condensed.
    // The first column is the name of the rule (which shows up in settings).
    // The second column is the incorrect form of the word and the third column is the correct
    // form.
    add_compound_mappings!(group, {
        "Anybody"         => &[(&["any body"], "anybody")],
        "Anyhow"          => &[(&["any how"], "anyhow")],
        "Anywhere"        => &[(&["any where"], "anywhere")],
        "Backplane"       => &[(&["back plane"], "backplane"),
                               (&["back planes"], "backplanes")],
        "Backport"        => &[(&["back port"], "backport"),
                               (&["back ported"], "backported"),
                               (&["back porting"], "backporting"),
                               (&["back ports"], "backports")],
        "Bypass"          => &[(&["by pass"], "bypass"),
                               (&["by passed"], "bypassed"),
                               (&["by passing"], "bypassing"),
                               (&["by passes"], "bypasses")],
        "Chalkboard"      => &[(&["chalk board"], "chalkboard"),
                               (&["chalk boards"], "chalkboards")],
        "Codebase"        => &[(&["code base"], "codebase"),
                               (&["code bases"], "codebases")],
        "Deadlift"        => &[(&["dead lift"], "deadlift"),
                               (&["dead lifts"], "deadlifts")],
        "Desktop"         => &[(&["desk top"], "desktop"),
                               (&["desk tops"], "desktops")],
        "Devops"          => &[(&["dev ops"], "devops")],
        "Everybody"       => &[(&["every body"], "everybody")],
        "Everyone"        => &[(&["every one"], "everyone")],
        "Everywhere"      => &[(&["every where"], "everywhere")],
        "Furthermore"     => &[(&["further more"], "furthermore")],
        "Henceforth"      => &[(&["hence forth"], "henceforth")],
        "However"         => &[(&["how ever"], "however")],
        "Insofar"         => &[(&["in so far"], "insofar")],
        "Instead"         => &[(&["in stead"], "instead")],
        "Intact"          => &[(&["in tact"], "intact")],
        "Itself"          => &[(&["it self"], "itself")],
        "Keystroke"       => &[(&["key stoke", "key stroke"], "keystroke"),
                               (&["key stokes", "key strokes"], "keystrokes")],
        "Laptop"          => &[(&["lap top"], "laptop"),
                               (&["lap tops"], "laptops")],
        "Middleware"      => &[(&["middle ware"], "middleware")],
        "Meanwhile"       => &[(&["mean while"], "meanwhile")],
        "Misadventure"    => &[(&["miss adventure"], "misadventure"),
                               (&["miss adventures"], "misadventures")],
        "Misshape"        => &[(&["miss shape"], "misshape"),
                               (&["miss shaped"], "misshaped"),
                               (&["miss shapen"], "misshapen"),
                               (&["miss shaping"], "misshaping"),
                               (&["miss shapes"], "misshapes")],
        "Misspeak"        => &[(&["miss speak"], "misspeak"),
                               (&["miss speaks"], "misspeaks"),
                               (&["miss speaking"], "misspeaking"),
                               (&["miss spoke"], "misspoke"),
                               (&["miss spoken"], "misspoken")],
        "Misspell"        => &[(&["miss spell", "miss-spell"], "misspell"),
                               (&["miss spells", "miss-spells"], "misspells"),
                               (&["miss spelled", "miss-spelled"], "misspelled"),
                               (&["miss spelling", "miss-spelling"], "misspelling"),
                               (&["miss spellings", "miss-spellings"], "misspellings")],
        "Misspend"        => &[(&["miss spend"], "misspend"),
                               (&["miss spends"], "misspends"),
                               (&["miss spending"], "misspending"),
                               (&["miss spent"], "misspent")],
        "Misstate"        => &[(&["miss state"], "misstate"),
                               (&["miss states"], "misstates"),
                               (&["miss stating"], "misstating"),
                               (&["miss stated"], "misstated"),
                               (&["miss statement"], "misstatement"),
                               (&["miss statements"], "misstatements")],
        "Misstep"         => &[(&["miss step"], "misstep"),
                               (&["miss steps"], "missteps"),
                               (&["miss stepping"], "misstepping"),
                               (&["miss stepped"], "misstepped")],
        "Mistake"         => &[(&["miss take"], "mistake"),
                               (&["miss takes"], "mistakes"),
                               (&["miss taking"], "mistaking"),
                               (&["miss took"], "mistook"),
                               (&["miss taken"], "mistaken"),],
        "Mistranslate"    => &[(&["miss translate"], "mistranslate"),
                               (&["miss translates"], "mistranslates"),
                               (&["miss translating"], "mistranslating"),
                               (&["miss translated"], "mistranslated"),
                               (&["miss translation"], "mistranslation"),
                               (&["miss translations"], "mistranslations")],
        "Mistime"         => &[(&["miss time"], "mistime"),
                               (&["miss times"], "mistimes"),
                               (&["miss timing"], "mistiming"),
                               (&["miss timed"], "mistimed")],
        "Mistreat"        => &[(&["miss treat"], "mistreat"),
                               (&["miss treated"], "mistreated"),
                               (&["miss treating"], "mistreating"),
                               (&["miss treats"], "mistreats"),
                               (&["miss treatment"], "mistreatment"),
                               (&["miss treatments"], "mistreatments")],
        "Mistrial"        => &[(&["miss trial"], "mistrial"),
                               (&["miss trials"], "mistrials")],
        "Mistrust"        => &[(&["miss trust"], "mistrust"),
                               (&["miss trusted"], "mistrusted"),
                               (&["miss trusting"], "mistrusting"),
                               (&["miss trusts"], "mistrusts"),
                               (&["miss trustful"], "mistrustful")],
        "Mistune"         => &[(&["miss tune"], "mistune"),
                               (&["miss tuned"], "mistuned"),
                               (&["miss tunes"], "mistunes"),
                               (&["miss tuning"], "mistuning")],
        "Mistype"         => &[(&["miss type"], "mistype"),
                               (&["miss typed"], "mistyped"),
                               (&["miss types"], "mistypes"),
                               (&["miss typing"], "mistyping")],
        "Misunderstand"   => &[(&["miss understand"], "misunderstand "),
                               (&["miss understanding"], "misunderstanding"),
                               (&["miss understands"], "misunderstands"),
                               (&["miss understood"], "misunderstood")],
        "Misuse"          => &[(&["miss use"], "misuse"),
                               (&["miss used"], "misused"),
                               (&["miss uses"], "misuses"),
                               (&["miss using"], "misusing")],
        "Multicore"       => &[(&["multi core"], "multicore")],
        "Multimedia"      => &[(&["multi media"], "multimedia")],
        "Multithreading"  => &[(&["multi threading"], "multithreading")],
        "Myself"          => &[(&["my self"], "myself")],
        "Nonetheless"     => &[(&["none the less"], "nonetheless")],
        "Nothing"         => &[(&["no thing"], "nothing")],
        "Notwithstanding" => &[(&["not with standing"], "notwithstanding")],
        "Nowhere"         => &[(&["no where"], "nowhere")],
        "Overall"         => &[(&["over all"], "overall")],
        "Overclocking"    => &[(&["over clocking"], "overclocking")],
        "Overload"        => &[(&["over load"], "overload"),
                               (&["over loaded"], "overloaded"),
                               (&["over loading"], "overloading"),
                               (&["over loads"], "overloads")],
        "Overnight"       => &[(&["over night"], "overnight"),
                               (&["over nights"], "overnights")],
        "Postpone"        => &[(&["post pone"], "postpone"),
                               (&["post poned"], "postponed"),
                               (&["post pones"], "postpones"),
                               (&["post poning"], "postponing")],
        "Proofread"       => &[(&["proof read"], "proofread"),
                               (&["proof reads"], "proofreads"),
                               (&["proof reading"], "proofreading"),
                               (&["proof reader"], "proofreader"),
                               (&["proof readers"], "proofreaders")],
        "Regardless"      => &[(&["regard less"], "regardless")],
        "Shortcoming"     => &[(&["short coming"], "shortcoming"),
                               (&["short comings"], "shortcomings")],
        "Somebody"        => &[(&["some body"], "somebody")],
        "Somehow"         => &[(&["some how"], "somehow")],
        "Someone"         => &[(&["some one"], "someone")],
        "Somewhere"       => &[(&["some where"], "somewhere")],
        "Tenfold"         => &[(&["ten fold"], "tenfold")],
        "There"           => &[(&["the re"], "there")],
        "Therefore"       => &[(&["there fore"], "therefore")],
        "Thereupon"       => &[(&["there upon"], "thereupon")],
        "Underclock"      => &[(&["under clock"], "underclock"),
                               (&["under clocked"], "underclocked"),
                               (&["under clocking"], "underclocking"),
                               (&["under clocks"], "underclocks")],
        "Upset"           => &[(&["up set"], "upset"),
                               (&["up sets"], "upsets")],
        "Upward"          => &[(&["up ward"], "upward"),
                               (&["up wards"], "upwards")],
        "Whereupon"       => &[(&["where upon"], "whereupon")],
        "Widespread"      => &[(&["wide spread"], "widespread")],
        "Without"         => &[(&["with out"], "without")],
        "Worldwide"       => &[(&["world wide"], "worldwide")],
        "Worthwhile"      => &[(&["worth while", "worth-while"], "worthwhile")],
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
    fn key_stoke() {
        let test_sentence = "Use this key stoke to open search.";
        let expected = "Use this keystroke to open search.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn in_tact() {
        let test_sentence = "The code remains in tact after the merge.";
        let expected = "The code remains intact after the merge.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn intact_is_allowed() {
        assert_no_lints("The data set remains intact.", lint_group());
    }

    #[test]
    fn key_stokes() {
        let test_sentence = "These key stokes are hard to memorize.";
        let expected = "These keystrokes are hard to memorize.";
        assert_suggestion_result(test_sentence, lint_group(), expected);
    }

    #[test]
    fn key_strokes() {
        let test_sentence = "There may have been a missing key stroke.";
        let expected = "There may have been a missing keystroke.";
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

    #[test]
    fn tenfold() {
        assert_suggestion_result(
            "If Andrew Kelly has done something wrong hasn’t Theo done that ten fold",
            lint_group(),
            "If Andrew Kelly has done something wrong hasn’t Theo done that tenfold",
        )
    }

    #[test]
    fn codebase() {
        assert_suggestion_result(
            "Having trouble communicating the problems in your code base?",
            lint_group(),
            "Having trouble communicating the problems in your codebase?",
        )
    }

    #[test]
    fn codebases() {
        assert_suggestion_result(
            "Tools to visualize large code bases in different ways.",
            lint_group(),
            "Tools to visualize large codebases in different ways.",
        )
    }

    #[test]
    fn backport() {
        assert_suggestion_result(
            "I have a few C++11 code bases (some back ported from newer versions)",
            lint_group(),
            "I have a few C++11 codebases (some backported from newer versions)",
        )
    }

    #[test]
    fn miss_adventure() {
        assert_suggestion_result(
            "The US is suffering from the myth of American Exceptionalism, from its finances to its military miss adventures.",
            lint_group(),
            "The US is suffering from the myth of American Exceptionalism, from its finances to its military misadventures.",
        )
    }

    #[test]
    fn base_form() {
        assert_suggestion_result(
            "They often miss spell names in the log.",
            lint_group(),
            "They often misspell names in the log.",
        );
    }

    #[test]
    fn past_tense() {
        assert_suggestion_result(
            "She miss spelled the answer on the quiz.",
            lint_group(),
            "She misspelled the answer on the quiz.",
        );
    }

    #[test]
    fn past_tense_hyphen() {
        assert_suggestion_result(
            "She miss-spelled the answer on the quiz.",
            lint_group(),
            "She misspelled the answer on the quiz.",
        );
    }

    #[test]
    fn gerund_form() {
        assert_suggestion_result(
            "His constant miss spelling frustrated the team.",
            lint_group(),
            "His constant misspelling frustrated the team.",
        );
    }
}
