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
}
