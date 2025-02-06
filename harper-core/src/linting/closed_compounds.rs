use crate::{
    patterns::{ExactPhrase, Pattern},
    Token, TokenStringExt,
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

macro_rules! create_closed_compound_linter {
    ($name:ident, $phrase:literal, $correct:expr) => {
        pub struct $name {
            pattern: Box<dyn Pattern>,
        }

        impl Default for $name {
            fn default() -> Self {
                let pattern = ExactPhrase::from_phrase($phrase);

                Self {
                    pattern: Box::new(pattern),
                }
            }
        }

        impl PatternLinter for $name {
            fn pattern(&self) -> &dyn Pattern {
                self.pattern.as_ref()
            }

            fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
                let span = matched_tokens.span().unwrap();
                let orig_chars = span.get_content(source);

                Lint {
                    span,
                    lint_kind: LintKind::WordChoice,
                    suggestions: vec![Suggestion::replace_with_match_case(
                        $correct.chars().collect(),
                        orig_chars,
                    )],
                    message: format!("Did you mean the closed compound `{}`?", $correct),
                    ..Default::default()
                }
            }

            fn description(&self) -> &'static str {
                concat!(
                    "Looks for incorrect spacing inside the closed compound `",
                    $correct,
                    "`."
                )
            }
        }
    };
}

create_closed_compound_linter!(Itself, "it self", "itself");
create_closed_compound_linter!(Tonight, "to night", "tonight");
create_closed_compound_linter!(Myself, "my self", "myself");
create_closed_compound_linter!(Therefore, "there fore", "therefore");
create_closed_compound_linter!(Misunderstand, "miss understand", "misunderstand");
create_closed_compound_linter!(Misunderstood, "miss understood", "misunderstood");
create_closed_compound_linter!(Misuse, "miss use", "misuse");
create_closed_compound_linter!(Misused, "miss used", "misused");
create_closed_compound_linter!(Postpone, "post pone", "postpone");
create_closed_compound_linter!(Worldwide, "world wide", "worldwide");
create_closed_compound_linter!(Overall, "over all", "overall");
create_closed_compound_linter!(Likewise, "like wise", "likewise");
create_closed_compound_linter!(However, "how ever", "however");
create_closed_compound_linter!(Altogether, "all together", "altogether");
create_closed_compound_linter!(Asleep, "a sleep", "asleep");
create_closed_compound_linter!(Upset, "up set", "upset");
create_closed_compound_linter!(Thumbnail, "thumb nail", "thumbnail");
create_closed_compound_linter!(Intact, "in tact", "intact");
create_closed_compound_linter!(Somehow, "some how", "somehow");
create_closed_compound_linter!(Bathroom, "bath room", "bathroom");
create_closed_compound_linter!(Facewash, "face wash", "facewash");
create_closed_compound_linter!(Handheld, "hand held", "handheld");
create_closed_compound_linter!(Playground, "play ground", "playground");
create_closed_compound_linter!(Runway, "run way", "runway");
create_closed_compound_linter!(Northeast, "north east", "northeast");
create_closed_compound_linter!(Northeastern, "north eastern", "northeastern");
create_closed_compound_linter!(Upholstery, "up holstery", "upholstery");
create_closed_compound_linter!(Proofread, "proof read", "proofread");

create_closed_compound_linter!(Forever, "for ever", "forever");
create_closed_compound_linter!(Into, "in to", "into");
create_closed_compound_linter!(Everyone, "every one", "everyone");
create_closed_compound_linter!(Nobody, "no body", "nobody");
create_closed_compound_linter!(Somebody, "some body", "somebody");
create_closed_compound_linter!(Anybody, "any body", "anybody");
create_closed_compound_linter!(Nothing, "no thing", "nothing");
create_closed_compound_linter!(Anyway, "any way", "anyway");
create_closed_compound_linter!(Anytime, "any time", "anytime");
create_closed_compound_linter!(Anywhere, "any where", "anywhere");
create_closed_compound_linter!(Nowhere, "no where", "nowhere");
create_closed_compound_linter!(Everywhere, "every where", "everywhere");
create_closed_compound_linter!(Overnight, "over night", "overnight");
create_closed_compound_linter!(Instead, "in stead", "instead");
create_closed_compound_linter!(Backpack, "back pack", "backpack");
create_closed_compound_linter!(Cupboard, "cup board", "cupboard");
create_closed_compound_linter!(Keyboard, "key board", "keyboard");
create_closed_compound_linter!(Somewhere, "some where", "somewhere");
create_closed_compound_linter!(Notebook, "note book", "notebook");
create_closed_compound_linter!(Middleware, "middle ware", "middleware");
create_closed_compound_linter!(Firmware, "firm ware", "firmware");
create_closed_compound_linter!(Smartwatch, "smart watch", "smartwatch");
create_closed_compound_linter!(Touchscreen, "touch screen", "touchscreen");
create_closed_compound_linter!(Headset, "head set", "headset");
create_closed_compound_linter!(Framework, "frame work", "framework");
create_closed_compound_linter!(Touchpad, "touch pad", "touchpad");
create_closed_compound_linter!(Microprocessor, "micro processor", "microprocessor");
create_closed_compound_linter!(Datacenter, "data center", "datacenter");
create_closed_compound_linter!(Smartphone, "smart phone", "smartphone");
create_closed_compound_linter!(Webcam, "web cam", "webcam");
create_closed_compound_linter!(Headphone, "head phone", "headphone");
create_closed_compound_linter!(Desktop, "desk top", "desktop");
create_closed_compound_linter!(Laptop, "lap top", "laptop");
create_closed_compound_linter!(Overclocking, "over clocking", "overclocking");
create_closed_compound_linter!(Backplane, "back plane", "backplane");
create_closed_compound_linter!(Smartcard, "smart card", "smartcard");
create_closed_compound_linter!(Bitrate, "bit rate", "bitrate");
create_closed_compound_linter!(Overload, "over load", "overload");
create_closed_compound_linter!(Underclock, "under clock", "underclock");
create_closed_compound_linter!(Blockchain, "block chain", "blockchain");
create_closed_compound_linter!(Opensource, "open source", "opensource");
create_closed_compound_linter!(Devops, "dev ops", "devops");
create_closed_compound_linter!(Bitstream, "bit stream", "bitstream");
create_closed_compound_linter!(Firewall, "fire wall", "firewall");
create_closed_compound_linter!(Sitemap, "site map", "sitemap");
create_closed_compound_linter!(Multithreading, "multi threading", "multithreading");
create_closed_compound_linter!(Multicore, "multi core", "multicore");
create_closed_compound_linter!(Microservices, "micro services", "microservices");
create_closed_compound_linter!(Dashboard, "dash board", "dashboard");
create_closed_compound_linter!(Cyberspace, "cyber space", "cyberspace");
create_closed_compound_linter!(Multimedia, "multi media", "multimedia");
create_closed_compound_linter!(Ecommerce, "e commerce", "ecommerce");
create_closed_compound_linter!(Datamining, "data mining", "datamining");
create_closed_compound_linter!(Datascience, "data science", "datascience");
create_closed_compound_linter!(Cyberattack, "cyber attack", "cyberattack");
create_closed_compound_linter!(Websocket, "web socket", "websocket");
create_closed_compound_linter!(Fingerprint, "finger print", "fingerprint");
create_closed_compound_linter!(Widespread, "wide spread", "widespread");
create_closed_compound_linter!(Notwithstanding, "not with standing", "notwithstanding");
create_closed_compound_linter!(Anyhow, "any how", "anyhow");
create_closed_compound_linter!(Nonetheless, "none the less", "nonetheless");
create_closed_compound_linter!(Hereafter, "here after", "hereafter");
create_closed_compound_linter!(Otherwise, "other wise", "otherwise");
create_closed_compound_linter!(Therein, "there in", "therein");
create_closed_compound_linter!(Thereupon, "there upon", "thereupon");
create_closed_compound_linter!(Hereby, "here by", "hereby");
create_closed_compound_linter!(Hereunder, "here under", "hereunder");
create_closed_compound_linter!(Forthwith, "forth with", "forthwith");
create_closed_compound_linter!(Insofar, "in so far", "insofar");
create_closed_compound_linter!(Whereupon, "where upon", "whereupon");
create_closed_compound_linter!(Thereafter, "there after", "thereafter");
create_closed_compound_linter!(Downright, "down right", "downright");
create_closed_compound_linter!(Upward, "up ward", "upward");
create_closed_compound_linter!(Henceforth, "hence forth", "henceforth");
create_closed_compound_linter!(Regardless, "regard less", "regardless");
create_closed_compound_linter!(Evermore, "ever more", "evermore");

#[cfg(test)]
mod tests {
    use super::{
        Altogether, Asleep, However, Itself, Likewise, Misunderstood, Misuse, Misused, Myself,
        Overall, Therefore, Tonight, Worldwide,
    };
    use super::{
        Anyhow, Downright, Evermore, Forthwith, Henceforth, Hereafter, Hereby, Hereunder, Insofar,
        Nonetheless, Notwithstanding, Otherwise, Regardless, Thereafter, Therein, Thereupon,
        Upward, Whereupon, Widespread,
    };
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn it_self() {
        let test_sentence = "The project, it self, was quite challenging.";
        let expected = "The project, itself, was quite challenging.";
        assert_suggestion_result(test_sentence, Itself::default(), expected);
    }

    #[test]
    fn to_night() {
        let test_sentence = "She spent the night to night.";
        let expected = "She spent the night tonight.";
        assert_suggestion_result(test_sentence, Tonight::default(), expected);
    }

    #[test]
    fn my_self() {
        let test_sentence = "He treated my self with respect.";
        let expected = "He treated myself with respect.";
        assert_suggestion_result(test_sentence, Myself::default(), expected);
    }

    #[test]
    fn there_fore() {
        let test_sentence = "This is the reason; there fore, this is true.";
        let expected = "This is the reason; therefore, this is true.";
        assert_suggestion_result(test_sentence, Therefore::default(), expected);
    }

    #[test]
    fn mis_understood() {
        let test_sentence = "She miss understood the instructions.";
        let expected = "She misunderstood the instructions.";
        assert_suggestion_result(test_sentence, Misunderstood::default(), expected);
    }

    #[test]
    fn mis_use() {
        let test_sentence = "He tends to miss use the tool.";
        let expected = "He tends to misuse the tool.";
        assert_suggestion_result(test_sentence, Misuse::default(), expected);
    }

    #[test]
    fn mis_used() {
        let test_sentence = "The software was miss used.";
        let expected = "The software was misused.";
        assert_suggestion_result(test_sentence, Misused::default(), expected);
    }

    #[test]
    fn world_wide() {
        let test_sentence = "The world wide impact was significant.";
        let expected = "The worldwide impact was significant.";
        assert_suggestion_result(test_sentence, Worldwide::default(), expected);
    }

    #[test]
    fn over_all() {
        let test_sentence = "The over all performance was good.";
        let expected = "The overall performance was good.";
        assert_suggestion_result(test_sentence, Overall::default(), expected);
    }

    #[test]
    fn like_wise() {
        let test_sentence = "He acted, like wise, without hesitation.";
        let expected = "He acted, likewise, without hesitation.";
        assert_suggestion_result(test_sentence, Likewise::default(), expected);
    }

    #[test]
    fn how_ever() {
        let test_sentence = "This is true, how ever, details matter.";
        let expected = "This is true, however, details matter.";
        assert_suggestion_result(test_sentence, However::default(), expected);
    }

    #[test]
    fn all_together() {
        let test_sentence = "They did it all together.";
        let expected = "They did it altogether.";
        assert_suggestion_result(test_sentence, Altogether::default(), expected);
    }

    #[test]
    fn a_sleep() {
        let test_sentence = "She fell a sleep.";
        let expected = "She fell asleep.";
        assert_suggestion_result(test_sentence, Asleep::default(), expected);
    }

    #[test]
    fn wide_spread() {
        let test_sentence = "The news was wide spread throughout the region.";
        let expected = "The news was widespread throughout the region.";
        assert_suggestion_result(test_sentence, Widespread::default(), expected);
    }

    #[test]
    fn not_with_standing() {
        let test_sentence = "They decided to proceed not with standing any further delay.";
        let expected = "They decided to proceed notwithstanding any further delay.";
        assert_suggestion_result(test_sentence, Notwithstanding::default(), expected);
    }

    #[test]
    fn any_how() {
        let test_sentence = "She solved the problem any how, even under pressure.";
        let expected = "She solved the problem anyhow, even under pressure.";
        assert_suggestion_result(test_sentence, Anyhow::default(), expected);
    }

    #[test]
    fn none_the_less() {
        let test_sentence = "The results were disappointing, none the less, they continued.";
        let expected = "The results were disappointing, nonetheless, they continued.";
        assert_suggestion_result(test_sentence, Nonetheless::default(), expected);
    }

    #[test]
    fn here_after() {
        let test_sentence = "He promised to abide by the rules here after the meeting.";
        let expected = "He promised to abide by the rules hereafter the meeting.";
        assert_suggestion_result(test_sentence, Hereafter::default(), expected);
    }

    #[test]
    fn other_wise() {
        let test_sentence = "Review the guidelines, other wise you might miss an important detail.";
        let expected = "Review the guidelines, otherwise you might miss an important detail.";
        assert_suggestion_result(test_sentence, Otherwise::default(), expected);
    }

    #[test]
    fn there_in() {
        let test_sentence = "All the answers can be found there in the document.";
        let expected = "All the answers can be found therein the document.";
        assert_suggestion_result(test_sentence, Therein::default(), expected);
    }

    #[test]
    fn there_upon() {
        let test_sentence = "A decision was made there upon reviewing the data.";
        let expected = "A decision was made thereupon reviewing the data.";
        assert_suggestion_result(test_sentence, Thereupon::default(), expected);
    }

    #[test]
    fn here_by() {
        let test_sentence = "The contract is here by declared null and void.";
        let expected = "The contract is hereby declared null and void.";
        assert_suggestion_result(test_sentence, Hereby::default(), expected);
    }

    #[test]
    fn here_under() {
        let test_sentence = "All terms are set forth here under.";
        let expected = "All terms are set forth hereunder.";
        assert_suggestion_result(test_sentence, Hereunder::default(), expected);
    }

    #[test]
    fn forth_with() {
        let test_sentence = "Please reply forth with to our previous inquiry.";
        let expected = "Please reply forthwith to our previous inquiry.";
        assert_suggestion_result(test_sentence, Forthwith::default(), expected);
    }

    #[test]
    fn in_so_far() {
        let test_sentence = "This rule applies in so far as it covers all cases.";
        let expected = "This rule applies insofar as it covers all cases.";
        assert_suggestion_result(test_sentence, Insofar::default(), expected);
    }

    #[test]
    fn where_upon() {
        let test_sentence = "They acted where upon the circumstances allowed.";
        let expected = "They acted whereupon the circumstances allowed.";
        assert_suggestion_result(test_sentence, Whereupon::default(), expected);
    }

    #[test]
    fn there_after() {
        let test_sentence = "The system shutdown occurred there after the update.";
        let expected = "The system shutdown occurred thereafter the update.";
        assert_suggestion_result(test_sentence, Thereafter::default(), expected);
    }

    #[test]
    fn down_right() {
        let test_sentence = "His comment was down right insulting to everyone present.";
        let expected = "His comment was downright insulting to everyone present.";
        assert_suggestion_result(test_sentence, Downright::default(), expected);
    }

    #[test]
    fn up_ward() {
        let test_sentence = "The temperature moved up ward during the afternoon.";
        let expected = "The temperature moved upward during the afternoon.";
        assert_suggestion_result(test_sentence, Upward::default(), expected);
    }

    #[test]
    fn hence_forth() {
        let test_sentence = "All new policies apply hence forth immediately.";
        let expected = "All new policies apply henceforth immediately.";
        assert_suggestion_result(test_sentence, Henceforth::default(), expected);
    }

    #[test]
    fn regard_less() {
        let test_sentence = "The decision was made, regard less of the opposition.";
        let expected = "The decision was made, regardless of the opposition.";
        assert_suggestion_result(test_sentence, Regardless::default(), expected);
    }
}
