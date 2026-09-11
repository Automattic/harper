use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, LongestMatchOf, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct HeadOffHeed {
    expr: LongestMatchOf,
}

impl Default for HeadOffHeed {
    fn default() -> Self {
        Self {
            expr: LongestMatchOf::new([
                Box::new(
                    // head of/heads of/heading of : legit
                    SequenceExpr::word_seq(&["headed", "of"]),
                ) as Box<dyn Expr>,
                Box::new(
                    // take.v* heed of : legit
                    SequenceExpr::word_set(&["heeded", "heeding", "heeds"])
                        .t_ws()
                        .t_aco("of"),
                ),
                Box::new(
                    // all wrong, no exceptions
                    SequenceExpr::word_set(&["heed", "heeded", "heeding", "heeds"])
                        .t_ws()
                        .t_aco("off"),
                ),
                Box::new(
                    SequenceExpr::word_set(&["take", "taken", "takes", "taking", "took"])
                        .t_ws()
                        .t_aco("head")
                        .t_ws()
                        .t_set(&["of", "off"]),
                ),
                Box::new(
                    SequenceExpr::word_set(&["take", "taken", "takes", "taking", "took"])
                        .t_ws()
                        .then_word_seq(&["heed", "off"]),
                ),
            ]),
        }
    }
}

#[derive(PartialEq)]
enum HeadOrHeed {
    Head,
    Heed,
}
use HeadOrHeed::*;

impl HeadOrHeed {
    fn new(s: &[char]) -> Option<Self> {
        if s.eq_str("heed") {
            Some(Heed)
        } else if s.eq_str("head") {
            Some(Head)
        } else {
            None
        }
    }
}

#[derive(PartialEq)]
enum OfOrOff {
    Of,
    Off,
}
use OfOrOff::*;

impl OfOrOff {
    fn new(s: &[char]) -> Option<Self> {
        if s.eq_str("of") {
            Some(Of)
        } else if s.eq_str("off") {
            Some(Off)
        } else {
            None
        }
    }
}

impl HeadOffHeed {
    fn handle_two_word_patterns(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // headed of
        // head-ed-ing-s of
        // heed-* off
        eprintln!(" 2️⃣");

        let (ht, ot) = (&toks[0], &toks[2]);

        let (h, o) = (
            HeadOrHeed::new(ht.get_ch(src))?,
            OfOrOff::new(ot.get_ch(src))?,
        );

        match (h, o) {
            (Head, Of) => {
                eprintln!("  2️⃣ 👤 OF 👉");
                None
            }
            (Head, Off) => {
                eprintln!("  2️⃣ 👤 📴 👉");
                None
            }
            (Heed, Of) => {
                eprintln!("  2️⃣ HEED OF 👉");
                None
            }
            (Heed, Off) => {
                eprintln!("  2️⃣ HEED 📴 👉");
                let span = ht.span;
                let lint_kind = LintKind::Usage;
                let suggs = ["head", "ward"];
                let suggestions = suggs
                    .iter()
                    .map(|s| Suggestion::replace_with_match_case_str(s, span.get_content(src)))
                    .collect();
                let message = "Did you mean `head off` (avoid upcoming danger) or `ward off` (defend against evil, disease, etc.)?".to_owned();
                Some(Lint {
                    span,
                    lint_kind,
                    suggestions,
                    message,
                    ..Default::default()
                })
            }
        }
    }

    fn handle_three_word_patterns(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // take-* head of/off
        // take-* heed off
        eprintln!(" 3️⃣");

        // The `Expr` must be allowing extra words
        if !toks[0]
            .get_ch(src)
            .eq_any_ignore_ascii_case_str(&["take", "takes", "taking", "took", "taken"])
        {
            return None;
        }

        let (ht, ot) = (&toks[2], &toks[4]);
        let (h, o) = (
            HeadOrHeed::new(ht.get_ch(src))?,
            OfOrOff::new(ot.get_ch(src))?,
        );

        match (h, o) {
            (HeadOrHeed::Head, OfOrOff::Of) => {
                eprintln!("  3️⃣ TAKE HEAD OF 👉");
                let span = ht.span;
                let lint_kind = LintKind::Typo;
                let suggs = ["heed", "the head"];
                let suggestions = suggs
                    .iter()
                    .map(|s| Suggestion::replace_with_match_case_str(s, span.get_content(src)))
                    .collect();
                let message = "Did you mean `take heed of` (listen to advice) or `take the head of` (a list, collection, etc.)?".to_owned();
                Some(Lint {
                    span,
                    lint_kind,
                    suggestions,
                    message,
                    ..Default::default()
                })
            }
            (HeadOrHeed::Head, OfOrOff::Off) => {
                eprintln!("  3️⃣ TAKE HEAD 📴 👉");
                None
            }
            (HeadOrHeed::Heed, OfOrOff::Of) => {
                eprintln!("  3️⃣ TAKE HEED OF 👉");
                None
            }
            (HeadOrHeed::Heed, OfOrOff::Off) => {
                eprintln!("  3️⃣ TAKE HEED 📴 👉");
                let span = ot.span;
                let lint_kind = LintKind::Typo;
                let suggs = ["of"];
                let suggestions = suggs
                    .iter()
                    .map(|s| Suggestion::replace_with_match_case_str(s, span.get_content(src)))
                    .collect();
                let message = "Did you mean `take heed of` (listen to advice)?".to_owned();
                Some(Lint {
                    span,
                    lint_kind,
                    suggestions,
                    message,
                    ..Default::default()
                })
            }
        }
    }
}

impl ExprLinter for HeadOffHeed {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));
        // two or three words (three of five) tokens?
        match toks.len() {
            3 => self.handle_two_word_patterns(toks, src),
            5 => self.handle_three_word_patterns(toks, src),
            _ => None,
        }
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter skeleton for contributors to copy into `harper_core/src/linting/` and rename."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_suggestion_result};

    use super::HeadOffHeed;

    // True positives

    // ‘heed off’ should be ‘head off’ (or maybe ‘ward off’?):

    #[test]
    fn fix_head_off_to_head_off_in_yozakura_family() {
        assert_suggestion_result(
            "The Yozakura Family have to get stronger, way stronger to heed off this threat, so we'll see the final results of their training next time!",
            HeadOffHeed::default(),
            "The Yozakura Family have to get stronger, way stronger to head off this threat, so we'll see the final results of their training next time!",
        );
    }

    #[test]
    fn fix_head_off_to_head_off_in_essentially() {
        assert_suggestion_result(
            "Essentially, you heed off their antisemitism by performatively cutting off ties with Israel",
            HeadOffHeed::default(),
            "Essentially, you head off their antisemitism by performatively cutting off ties with Israel",
        );
    }

    /*
    - just to **heed off** future responses, when we take outside food (or alcohol) from customers
    - I'm hoping it'll at least **heed off** some complaints down the line.
    - To **heed off** this trend the Faculty is seeking to hire new high-quality researchers.
    - Despite this, they are fighting tooth and nail to **heed off** the threat.
    - what road to take to **heed off** this increasingly alarming emergency
    - Where others have gone to ignorance to **heed off** such pain, I had no such ability.
    - people who can successfuly **heed off** even prediabetes from happening with lifestyle alone
    - Is to protect other refs and try **heed off** discussions about refs making decisions with Bias.
    - Despite this, they are fighting tooth and nail to **heed off** the threat.
    - Any of these options could have **heeded off** the potential of him escaping
    - Or should I send an email to GSI to try to **heed off** any potential negative impact now before it happens?
    - get yourself in front of a client and **heed off** that discontent
    - And, just to **heed off** replies, I do not believe having the ...
    - without anything bigger than a Gozanti to **heed off** the Corvette's we're going to struggle there
    - if you are serious about infrastructure investments to **heed off** this recession
    - I've tried to dream feed around 9:45/10pm to **heed off** the first wakeup
    - They committed to lockdowns in March to **heed off** a poorly known disease
    - Unifies both Targaryen claims under one heir, which will **heed off** an immediate succession crisis
    - doing their due diligence to **heed off** problems before they occur
    - The good news is that there are many ways we can act to **heed off** this future.
    - exhausted and barely able to **heed off** flare ups of alopecia areata
    - trying to **heed off** a potential conflict flashpoint.
    - to **heed off** the next argument: there was a debate amongst tye rabbis
    */

    // ‘heed’ is a (possibly intentional humorous dialectal?) misspelling of ‘head’:

    #[test]
    fn fix_head_off_to_head_off_in_spicy_lemon_sage_tongue_tingler() {
        assert_suggestion_result(
            "spicy lemon/sage tongue tingler taste lol blew the heed off me too it was fantastic",
            HeadOffHeed::default(),
            "spicy lemon/sage tongue tingler taste lol blew the head off me too it was fantastic",
        );
    }

    /*
    - Feth, he was like to ha' snapped the **heed off** me
    - ive been ripping the **heed off** it for more than that mate
    - when I stand up, they're concerned for my safety in case the ball takes my **heed off**
    - You may want to try this to help **heed off** dehydration.
    - it is called Big Climb and it will blow your fucking **heed off**
    - I was toting one as I **heed off** to the headwaters of Tagoloan River in Malaybalay City for the Panendan
    - We **heed off** to the punt Regatta.
    - they're going to want to rip each others **heed off**, right?!?
    - Smacking your **heed off** a wall is best done with a helmet on as I found out!
    - I hope Usyk boxes his **heed off**.
    - Im all for cleaning up the El, and getting the crack **heeds off** the street
    - Felt like the helmet was gonna rip my **heed off** whilst checking my mirrors.
    - Aal pure borst ya lips And blaa ya **heed off**
    - Thanks to my youngest screaming his **heed off** for a few hours the other night
    - its impossible not to just smile yer' **heed off** in their presence
    - Increases str and AP but it also has a chance to blow your **heed off**
    */

    // ‘heed off’ should be ‘heed of’:

    #[test]
    fn fix_take_heed_off_to_take_heed_of() {
        assert_suggestion_result(
            "A reminder everyone needs to take heed off, no matter the job.",
            HeadOffHeed::default(),
            "A reminder everyone needs to take heed of, no matter the job.",
        );
    }

    /*
    - lessons that social media companies didn’t take **heed off** in the first ten years of their lives
    - I mostly took **heed off** of madness combat due to thy wasteland/grimy feelings to it.
    - A reminder everyone needs to take **heed off**, no matter the job.
    - I just hope y'all take **heed off** my example and avoid these lol.
    - His response was one which we should all take **heed off**
    - And this is something that we not even taking **heed off**.
    - I'll take **heed off** your advice.
    - You may refuse, **heed off** warnings from friends, family members 👉 take heed of
    - And so what is it that God's saying to you today that you might need to take **heed off** that you don't end up in a place worse off in the future?
    */

    // ‘heed off’ should be ‘ward off’

    #[test]
    fn fix_heed_off_to_ward_off() {
        assert_suggestion_result(
            "makes me think the human made these… dolls to heed off predators in the wild",
            HeadOffHeed::default(),
            "makes me think the human made these… dolls to ward off predators in the wild",
        );
    }

    /*
    - blue is a color used for protection and **heeds off** negative energy
    - They work like a charm, inexpensive and help **heed off** odors too.
    - Back when illness could be **heeded off** or cured by amulets or masked dances
    - Sour kraut, beet kavass, etc you use salt or whey to **heed off** the bad bugs and let the natural enzymes ferment it.
    */

    // take-* ‘head of’ should be take-* ‘heed of’:

    #[test]
    fn fix_head_of_to_heed_of() {
        assert_suggestion_result(
            "The consumers of the initial release had better take head of the phrases \"Anything MAY change at any time.",
            HeadOffHeed::default(),
            "The consumers of the initial release had better take heed of the phrases \"Anything MAY change at any time.",
        );
    }

    /*- Android Studion - theyve really taken **head of** developers.
    - Most engineers should take **head of** reason three
    */

    // Potential false positives to avoid

    // ‘take head of’ is just shorthand for ‘take the head of’

    #[test]
    fn dont_flag_list() {
        assert_suggestion_result(
            "cannot take head of empty list",
            HeadOffHeed::default(),
            "cannot take the head of empty list",
        );
    }

    /*- **take head of** queue and prepend to parse queue
    - Added the function corner() which **takes head of** rows and columns of objects.
    - **take head of** the tree and traverse it
    - Can't **take head of** empty collection
    - ;; else **take head of** list
    - it **takes head of** List and prints the linked list in output terminal
    - bind takes the whole argument sent,; first **takes head of** the argument
    - I **take head of** original list by value and assign it to L
    - **Take head of** the list; Compare head with remaining N-1 elements in the list;
    */

    // Too ambiguous to confidently classify:

    #[test]
    fn dont_flag_ambiguous() {
        // Take heed of the value of that? Or take the head of some kind of 'base'?
        assert_good_and_bad_suggestions(
            "Flutter tooling should take head of archivesBaseName",
            HeadOffHeed::default(),
            &[
                "Flutter tooling should take heed of archivesBaseName",
                "Flutter tooling should take the head of archivesBaseName",
            ],
            &[],
        );
    }

    /*- if it's a nested Node, **take head of** Node's content 👉 take heed of? / take the head of?
     */
}
