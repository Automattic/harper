use crate::linting::LintGroup;

use super::InitialismLinter;

pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::empty();

    macro_rules! add_initialism_mappings {
        ($group:expr, { $($name:expr => ($initialism:expr, $expanded:expr)),+ $(,)? }) => {
            $(
                $group.add_expr_linter(
                    $name,
                    Box::new(InitialismLinter::new($initialism, $expanded)),
                );
            )+
        };
    }

    add_initialism_mappings!(group, {
        "ByTheWay"           => ("btw", "by the way"),
        "ForYourInformation" => ("fyi", "for your information"),
        "AsSoonAsPossible"   => ("asap", "as soon as possible"),
        "InMyOpinion"        => ("imo", "in my opinion"),
        "InMyHumbleOpinion"  => ("imho", "in my humble opinion"),
        "OhMyGod"            => ("omg", "oh my god"),
        "BeRightBack"        => ("brb", "be right back"),
        "TalkToYouLater"     => ("ttyl", "talk to you later"),
        "NeverMind"          => ("nvm", "never mind"),
        "ToBeHonest"         => ("tbh", "to be honest"),
        "AsFarAsIKnow"       => ("afaik", "as far as I know"),
        "Really"             => ("rly", "really"),
        "ExplainLikeImFive"  => ("eli5", "explain like i'm five"),
        "ForWhatItsWorth"    => ("fwiw", "for what it's worth"),
        "IDontKnow"          => ("idk", "I don't know"),
        "IfIRecallCorrectly" => ("iirc", "if I recall correctly"),
        "IfYouKnowYouKnow"   => ("iykyk", "if you know, you know"),
        "InCaseYouMissedIt"  => ("icymi", "in case you missed it"),
        "InRealLife"         => ("irl", "in real life"),
        "PleaseTakeALook"    => ("ptal", "please take a look"),
    });

    group.set_all_rules_to(Some(true));

    group
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::lint_group;

    #[test]
    fn corrects_btw() {
        assert_suggestion_result(
            "Btw, are you ready to go shopping soon?",
            lint_group(),
            "By the way, are you ready to go shopping soon?",
        );
    }

    #[test]
    fn corrects_style() {
        assert_suggestion_result(
            "I love the fit, btw.",
            lint_group(),
            "I love the fit, by the way.",
        );
    }

    #[test]
    fn corrects_fyi() {
        assert_suggestion_result(
            "Fyi, the meeting is at 3.",
            lint_group(),
            "For your information, the meeting is at 3.",
        );
    }

    #[test]
    fn corrects_asap() {
        assert_suggestion_result(
            "Please respond asap.",
            lint_group(),
            "Please respond as soon as possible.",
        );
    }

    #[test]
    fn corrects_imo() {
        assert_suggestion_result(
            "Imo, that is the best option.",
            lint_group(),
            "In my opinion, that is the best option.",
        );
    }

    #[test]
    fn corrects_omg() {
        assert_suggestion_result(
            "Omg! That's incredible!",
            lint_group(),
            "Oh my god! That's incredible!",
        );
    }

    #[test]
    fn corrects_brb() {
        assert_suggestion_result("Hold on, brb.", lint_group(), "Hold on, be right back.");
    }

    #[test]
    fn corrects_tbh() {
        assert_suggestion_result(
            "Tbh, I'm not impressed.",
            lint_group(),
            "To be honest, I'm not impressed.",
        );
    }

    #[test]
    fn corrects_rly() {
        assert_suggestion_result(
            "Rly excited for this.",
            lint_group(),
            "Really excited for this.",
        );
    }

    #[test]
    fn issue_2181() {
        assert_suggestion_result(
            "AFAIK, we don't currently have an issue for it.",
            lint_group(),
            "As far as i know, we don't currently have an issue for it.",
        );
    }

    #[test]
    fn corrects_eli5() {
        assert_suggestion_result(
            "Can you eli5 how this works?",
            lint_group(),
            "Can you explain like i'm five how this works?",
        );
    }

    #[test]
    fn corrects_fwiw() {
        assert_suggestion_result(
            "Fwiw, I think it's a good idea.",
            lint_group(),
            "For what it's worth, I think it's a good idea.",
        );
    }

    #[test]
    fn corrects_idk() {
        assert_suggestion_result(
            "Idk if I'll make it to the party.",
            lint_group(),
            "I don't know if I'll make it to the party.",
        );
    }

    #[test]
    fn corrects_iirc() {
        assert_suggestion_result(
            "Iirc, the event starts at 6 PM.",
            lint_group(),
            "If i recall correctly, the event starts at 6 PM.",
        );
    }

    #[test]
    fn corrects_iykyk() {
        assert_suggestion_result(
            "Iykyk, this place is amazing.",
            lint_group(),
            "If you know, you know, this place is amazing.",
        );
    }

    #[test]
    fn corrects_icymi() {
        assert_suggestion_result(
            "Icymi, the deadline is tomorrow.",
            lint_group(),
            "In case you missed it, the deadline is tomorrow.",
        );
    }

    #[test]
    fn corrects_irl() {
        assert_suggestion_result(
            "We should meet irl sometime.",
            lint_group(),
            "We should meet in real life sometime.",
        );
    }

    #[test]
    fn corrects_ptal() {
        assert_suggestion_result(
            "Ptal at the document I sent.",
            lint_group(),
            "Please take a look at the document I sent.",
        );
    }
}
