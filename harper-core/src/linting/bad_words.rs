use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::{LintKind, Suggestion};

use super::{ExprLinter, Lint};

pub struct BadWords {
    expr: Box<dyn Expr>,
}

impl Default for BadWords {
    fn default() -> Self {
        Self {
            expr: Box::new(SequenceExpr::default().then_swear()),
        }
    }
}

impl ExprLinter for BadWords {
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

        // Define offensive words and their possible replacements
        const CENSOR: &[(&str, &[&str])] = &[
            ("arse", &["bum", "backside", "bottom", "rump", "posterior"]),
            (
                "arses",
                &["bums", "backsides", "bottoms", "rumps", "posteriors"],
            ),
            ("arsed", &["bothered"]),
            ("arsehole", &["bumhole"]),
            ("ass", &["butt"]),
            ("asses", &["butts"]),
            ("asshole", &["butthole"]),
            ("batshit", &["batsh*t"]),
            ("birdshit", &["birdsh*t"]),
            (
                "bullshit",
                &["bullsh*t", "bullcrap", "bulldust", "lie", "lies"],
            ),
            ("bullshitted", &["bullsh*tted", "bullcrapped", "lied"]),
            ("bullshitting", &["bullsh*tting", "bullcrapping", "lying"]),
            ("bullshitter", &["bullsh*tter", "liar"]),
            // bullshittery
            // chickenshit
            ("cock", &["c*ck", "penis"]),
            ("cocks", &["c*cks", "penises"]),
            // cocksucker
            ("crap", &["poo", "poop", "feces", "dung"]),
            ("craps", &["poos", "poops", "feces", "dung"]),
            ("crapped", &["pooed", "pooped"]),
            ("crapping", &["pooing", "pooping"]),
            ("cunt", &["c*nt", "vagina"]),
            ("cunts", &["c*nts", "vaginas"]),
            ("crapload", &["cr*pload", "shedload"]),
            ("dick", &["d*ck", "penis"]),
            ("dicks", &["d*cks", "penises"]),
            ("dickhead", &["d*ckhead"]),
            ("dickheads", &["d*ckheads"]),
            ("dumbass", &["dumb*ss", "idiot"]),
            ("dumbasses", &["dumb*sses", "idiots"]),
            ("fart", &["gas", "wind", "break wind"]),
            ("farts", &["gas", "wind", "breaks wind"]),
            ("farted", &["broke wind", "broken wind"]),
            ("farting", &["break wind"]),
            ("fuck", &["f*ck", "fudge", "screw"]),
            ("fucks", &["f*cks", "screws"]),
            ("fucked", &["f*cked", "screwed"]),
            ("fucking", &["f*cking", "screwing"]),
            ("fucker", &["f*cker"]),
            ("fuckers", &["f*ckers"]),
            ("fuckhead", &["f*ckhead"]),
            ("fuckheads", &["f*ckheads"]),
            ("horseshit", &["horsesh*t"]),
            ("mindfuck", &["mindf*ck"]),
            ("motherfucker", &["motherf*cker"]),
            ("motherfuckers", &["motherf*ckers"]),
            ("motherfucking", &["motherf*cking"]),
            // nigga
            // nigger
            ("piss", &["p*ss", "pee"]),
            ("pisses", &["p*sses", "pees"]),
            ("pissed", &["p*ssed", "peed"]),
            (
                "pisser",
                &["p*sser", "toilet", "bathroom", "restroom", "washroom"],
            ),
            ("pissing", &["p*ssing", "peeing"]),
            // pissy
            ("shit", &["sh*t", "poo", "poop", "feces", "dung"]),
            ("shits", &["sh*ts", "craps", "poos", "poops"]),
            ("shitted", &["sh*tted", "crapped", "pooed", "pooped"]),
            ("shitting", &["sh*ting", "crapping", "pooing", "pooping"]),
            // shitcoin
            // shitfaced
            // shitfest
            // shithead
            // shitless
            (
                "shitload",
                &["sh*tload", "crapload", "shedload", "load", "tons", "pile"],
            ),
            (
                "shitloads",
                &[
                    "sh*tloads",
                    "craploads",
                    "shedloads",
                    "loads",
                    "tons",
                    "piles",
                ],
            ),
            // shitpost
            ("shitty", &["sh*tty", "shirty", "crappy"]),
            ("shittier", &["sh*ttier", "shirtier", "crappier"]),
            ("shittiest", &["sh*ttiest", "shirtiest", "crappiest"]),
            ("turd", &["poo", "poop", "feces", "dung"]),
            ("turds", &["poos", "poops", "feces", "dung"]),
            // twat
            ("wank", &["w*ank"]),
            ("wanks", &["w*anks"]),
            ("wanked", &["w*anked"]),
            ("wanking", &["w*anking"]),
            ("wanker", &["w*anker"]),
            ("wankers", &["w*ankers"]),
            ("wanky", &["w*anky"]),
            // whore
        ];

        // Find all replacement suggestions for the bad word
        let replacements: Vec<&str> = CENSOR
            .iter()
            .filter(|(bad, _)| *bad == bad_word_norm)
            .flat_map(|(_, suggestions)| suggestions.iter().copied())
            .collect();

        if replacements.is_empty() {
            return None;
        }

        // Create suggestions for each replacement
        let suggestions = replacements
            .iter()
            .map(|replacement| Suggestion::replace_with_match_case_str(replacement, bad_word_chars))
            .collect();

        // Create appropriate message
        let message = format!("You could use a nicer word like `{}`", replacements[0]);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Replaces bad words with nicer synonyms"
    }
}

#[cfg(test)]
mod tests {
    use super::BadWords;
    use crate::linting::tests::assert_top3_suggestion_result;

    #[test]
    fn fix_shit() {
        assert_top3_suggestion_result("shit", BadWords::default(), "poo")
    }

    #[test]
    fn fix_shit_titlecase() {
        assert_top3_suggestion_result("Shit", BadWords::default(), "Poo")
    }

    #[test]
    fn fix_shit_allcaps() {
        assert_top3_suggestion_result("SHIT", BadWords::default(), "POO")
    }
}
