use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::{LintKind, Suggestion};

use super::{ExprLinter, Lint};

pub struct AvoidCurses {
    expr: Box<dyn Expr>,
}

impl Default for AvoidCurses {
    fn default() -> Self {
        Self {
            expr: Box::new(SequenceExpr::default().then_swear()),
        }
    }
}

impl ExprLinter for AvoidCurses {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() != 1 {
            return None;
        }

        let tok = &toks[0];
        let span = tok.span;
        let bad_word_chars = span.get_content(src);
        let bad_word_str = span.get_content_string(src);
        let bad_word_norm = bad_word_str.to_lowercase();

        // Define offensive morphemes which are common parts of multiple words
        // Each entry maps a morpheme to an optional censored version.
        const MORPHEMES: &[(&str, Option<&str>)] = &[
            ("arse", None),
            ("ass", Some("a**")),
            ("cock", Some("c**k")),
            ("cunt", Some("c**t")),
            ("dick", Some("d**k")),
            ("fuck", Some("f**k")),
            ("piss", Some("p**s")),
            ("shit", Some("sh*t")),
            ("wank", Some("w**k")),
        ];

        // Define offensive words and their possible replacements
        const WORDS: &[(&str, &[&str])] = &[
            ("apeshit", &["crazy", "mad", "insane", "wild"]),
            (
                "arse",
                &["bum", "buttocks", "backside", "bottom", "rump", "posterior"],
            ),
            (
                "arses",
                &[
                    "bums",
                    "buttocks",
                    "backsides",
                    "bottoms",
                    "rumps",
                    "posteriors",
                ],
            ),
            ("arsed", &["bothered"]),
            ("arsehole", &["bumhole"]),
            (
                "ass",
                &[
                    "butt",
                    "buttocks",
                    "backside",
                    "bottom",
                    "rump",
                    "posterior",
                    "tuchus",
                    "tush",
                ],
            ),
            (
                "asses",
                &[
                    "butts",
                    "buttocks",
                    "backsides",
                    "bottoms",
                    "rumps",
                    "posteriors",
                    "tuchuses",
                    "tushes",
                ],
            ),
            ("asshole", &["butthole"]),
            // batshit
            // birdshit
            ("bullshit", &["bullcrap", "bulldust", "lie", "lies"]),
            ("bullshitted", &["bullcrapped", "lied"]),
            ("bullshitting", &["bullcrapping", "lying"]),
            ("bullshitter", &["liar"]),
            // bullshittery
            ("chickenshit", &["gutless", "cowardly"]),
            ("cock", &["pee-pee", "willy", "penis", "phallus", "member"]),
            (
                "cocks",
                &["pee-pees", "willies", "penises", "phalluses", "members"],
            ),
            // cocksucker
            ("cunt", &["vagina"]),
            ("cunts", &["vaginas"]),
            ("dick", &["pee-pee", "penis"]),
            ("dicks", &["pee-pees", "penises"]),
            ("dickhead", &["jerk", "idiot"]),
            ("dichheads", &["jerks", "idiots"]),
            // dipshit
            ("dumbass", &["idiot", "fool"]),
            ("dumbasses", &["idiots", "fools"]),
            ("fart", &["gas", "wind", "break wind"]),
            ("farts", &["gas", "wind", "breaks wind"]),
            ("farted", &["broke wind", "broken wind"]),
            ("farting", &["breaking wind"]),
            ("fuck", &["fudge", "screw", "damn", "hoot"]),
            ("fucks", &["screws"]),
            ("fucked", &["screwed"]),
            ("fucking", &["screwing"]),
            ("fucker", &["jerk"]),
            ("fuckers", &["jerks"]),
            // fuckhead
            ("horseshit", &["nonsense"]),
            // mindfuck
            // motherfucker
            // nigga
            // nigger
            ("piss", &["pee", "urine", "urinate"]),
            ("pisses", &["pees", "urinates"]),
            ("pissed", &["peed", "urinated"]),
            ("pissing", &["peeing", "urinating"]),
            ("pisser", &["toilet", "bathroom", "restroom", "washroom"]),
            // pissy
            (
                "shit",
                &["crap", "poo", "poop", "feces", "dung", "damn", "hoot"],
            ),
            ("shits", &["craps", "poos", "poops"]),
            ("shitted", &["crapped", "pooed", "pooped"]),
            ("shitting", &["crapping", "pooing", "pooping"]),
            // shitcoin
            // shitfaced
            // shitfest
            // shithead
            ("shitless", &["witless"]),
            (
                "shitload",
                &["crapload", "shedload", "shirtload", "load", "tons", "pile"],
            ),
            (
                "shitloads",
                &[
                    "craploads",
                    "shedloads",
                    "shirtloads",
                    "loads",
                    "tons",
                    "piles",
                ],
            ),
            // shitpost
            ("shitty", &["shirty", "crappy", "inferior"]),
            ("shittier", &["crappier", "shirtier"]),
            ("shittiest", &["crappiest", "shirtiest"]),
            ("tit", &["boob", "breast"]),
            ("tits", &["boobs", "breasts"]),
            ("titty", &["boob", "breast"]),
            ("titties", &["boobs", "breasts"]),
            ("turd", &["poo", "poop", "feces", "dung"]),
            ("turds", &["poos", "poops", "feces", "dung"]),
            ("twat", &["vagina"]),
            // wank
            ("wanker", &["jerk"]),
            // wanky
            ("whore", &["prostitute"]),
        ];

        // Replace common morphemes with both specific censored versions and all-asterisk versions
        let morpheme_replacements: Vec<String> = MORPHEMES
            .iter()
            .filter(|(m, _)| bad_word_norm.contains(m))
            .flat_map(|(m, censored)| {
                let mut replacements = Vec::new();

                // Add all-asterisk version for the censored morpheme only
                let asterisked = "*".repeat(m.len());
                let asterisked_word = bad_word_norm.replace(m, &asterisked);
                replacements.push(asterisked_word);

                // Add specific censored version if it exists
                if let Some(c) = censored {
                    let censored_word = bad_word_norm.replace(m, c);
                    replacements.push(censored_word);
                }

                replacements
            })
            .collect();

        // Find all replacement suggestions for the bad word
        let word_replacements: Vec<&str> = WORDS
            .iter()
            .filter(|(bad, _)| *bad == bad_word_norm)
            .flat_map(|(_, suggestions)| suggestions.iter().copied())
            .collect();

        if morpheme_replacements.is_empty() && word_replacements.is_empty() {
            return None;
        }

        let m_suggestions: Vec<Suggestion> = morpheme_replacements
            .into_iter()
            .map(|replacement| {
                Suggestion::replace_with_match_case(replacement.chars().collect(), bad_word_chars)
            })
            .collect();

        let w_suggestions: Vec<Suggestion> = word_replacements
            .into_iter()
            .map(|replacement| {
                Suggestion::replace_with_match_case(replacement.chars().collect(), bad_word_chars)
            })
            .collect();

        let suggestions = m_suggestions.into_iter().chain(w_suggestions).collect();

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message: "Try to avoid offensive language.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Flags offensive language and offers various ways to censor or replace with euphemisms."
    }
}

#[cfg(test)]
mod tests {
    use super::AvoidCurses;
    use crate::linting::tests::{assert_lint_count, assert_top3_suggestion_result};

    #[test]
    fn detects_shit() {
        assert_lint_count(
            "He ate shit when he fell off the bike.",
            AvoidCurses::default(),
            1,
        );
    }

    #[test]
    fn fix_shit() {
        assert_top3_suggestion_result("shit", AvoidCurses::default(), "crap")
    }

    #[test]
    fn fix_shit_titlecase() {
        assert_top3_suggestion_result("Shit", AvoidCurses::default(), "Crap")
    }

    #[test]
    fn fix_shit_allcaps() {
        assert_top3_suggestion_result("SHIT", AvoidCurses::default(), "CRAP")
    }

    #[test]
    fn fix_f_word_to_all_asterisks() {
        assert_top3_suggestion_result(
            "fuck those fucking fuckers",
            AvoidCurses::default(),
            "**** those ****ing ****ers",
        )
    }

    #[test]
    fn fix_shit_with_single_asterisk() {
        assert_top3_suggestion_result("shit", AvoidCurses::default(), "sh*t")
    }

    #[test]
    fn fix_shite_all_caps_with_single_asterisk() {
        assert_top3_suggestion_result("SHIT", AvoidCurses::default(), "SH*T")
    }
}
