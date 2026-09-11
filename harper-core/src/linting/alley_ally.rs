use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, FirstMatchOf, OwnedExprExt, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

// Words that normally precede "alley" but signal an error when used before "ally".
const ALLEY_PREMODIFIERS: &[&str] = &[
    "back",
    "blind",
    "bowling",
    // "damp",    // Rejected: too ambiguous with valid ally contexts
    // "dank",    // Rejected: too ambiguous with valid ally contexts
    "dark",
    "darkest",
    "dingy",
    "dingiest",
    // "dodgy",   // Rejected: too ambiguous with valid ally contexts
    "dodgiest",
    "narrow",
    // "sketchy", // Rejected: too ambiguous with valid ally contexts
    "sketchiest",
];

// Words that normally precede "ally" but signal an error when used before "alley".
const ALLY_PREMODIFIERS: &[&str] = &[
    // "closest",   // Rejected: too ambiguous with valid alley contexts
    "coalition",
    "faithful",
    // "key",
    "loyal",
    "loyalest",
    "nato",
    // "natural",   // Rejected: too ambiguous with valid alley contexts
    // "political", // Rejected: too ambiguous with valid alley contexts
    "potential",
    // "powerful",  // Rejected: too ambiguous with valid alley contexts
    "reliable",
    "staunch",
    "staunchest",
    // "strong",    // Rejected: too ambiguous with valid alley contexts
    "strongest",
    "useful",
];

pub struct AlleyAlly {
    expr: FirstMatchOf,
}

impl Default for AlleyAlly {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new(vec![
                // * alley -> * ally
                // Rejected pattern: "hidden" (too ambiguous)
                Box::new(
                    SequenceExpr::word_set(ALLEY_PREMODIFIERS)
                        .t_ws()
                        .t_set(&["ally", "allies"]),
                ) as Box<dyn Expr>,
                // ally * -> alley *
                // Rejected patterns: "along", "between" (too ambiguous)
                Box::new(
                    SequenceExpr::aco("ally")
                        .t_ws_h()
                        .t_set(&["cat", "cats", "oop", "oops", "way", "ways"]),
                ),
                // * alley -> * ally
                Box::new(
                    SequenceExpr::word_set(ALLY_PREMODIFIERS)
                        .t_ws()
                        .t_set(&["alley", "alleys"])
                        .but_not(FirstMatchOf::new([
                            Box::new(
                                SequenceExpr::anything()
                                    .t_any()
                                    .t_aco("alley")
                                    .t_ws_h()
                                    .t_aco("oop"),
                            ),
                            Box::new(
                                SequenceExpr::with(|t: &Token, s: &[char]| {
                                    t.get_ch(s)
                                        .first()
                                        .is_some_and(|ch| ch.is_ascii_uppercase())
                                })
                                .t_any()
                                .then_exact_word("Alley"),
                            ),
                        ])),
                ) as Box<dyn Expr>,
                // alley * -> ally *
            ]),
        }
    }
}

impl ExprLinter for AlleyAlly {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tok = find_the_only_token_matching(toks, src, |t, s| {
            t.get_ch(s)
                .eq_any_ignore_ascii_case_str(&["alley", "alleys", "ally", "allies"])
        })?;

        let ch = tok.get_ch(src);
        let fourth_letter = ch.get(3)?;

        let correction = match (fourth_letter, ch.len()) {
            // all[e]y -> ally
            ('e', 5) => "ally",
            // all[e]ys -> allies
            ('e', 6) => "allies",
            // all[i]es -> alleys
            ('i', 6) => "alleys",
            // all[y] -> alley
            ('y', 4) => "alley",
            _ => return None,
        };

        let span = tok.span;

        let suggestions = vec![Suggestion::replace_with_match_case_str(
            correction,
            span.get_content(src),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::Spelling,
            suggestions,
            message:
                "Did you mean ‘alley’ (a narrow street) or ‘ally’ (a partner with a shared cause)?"
                    .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects confusions between 'alley' and 'ally'."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AlleyAlly;

    // <adjective> "ally" with adjectives normally used with "alley".

    #[test]
    fn fix_back_ally() {
        assert_suggestion_result(
            "Bar fights and back ally duels will probably be fine using normal combat rules",
            AlleyAlly::default(),
            "Bar fights and back alley duels will probably be fine using normal combat rules",
        );
    }

    #[test]
    fn fix_back_allies() {
        assert_suggestion_result(
            "an epic read spanning continents, countries, prisons, back allies, jails, dope houses, remote villages, and city slums",
            AlleyAlly::default(),
            "an epic read spanning continents, countries, prisons, back alleys, jails, dope houses, remote villages, and city slums",
        );
    }

    #[test]
    fn fix_blind_ally() {
        assert_suggestion_result(
            "quite probable that it will turn into a blind ally and need to back itself out",
            AlleyAlly::default(),
            "quite probable that it will turn into a blind alley and need to back itself out",
        );
    }

    #[test]
    fn fix_blind_allies() {
        assert_suggestion_result(
            "they go off onto tangents, blind allies or just do the wrong thing",
            AlleyAlly::default(),
            "they go off onto tangents, blind alleys or just do the wrong thing",
        );
    }

    #[test]
    fn fix_bowling_ally() {
        assert_suggestion_result(
            "the end of the bowling ally where she frontflips over",
            AlleyAlly::default(),
            "the end of the bowling alley where she frontflips over",
        );
    }

    #[test]
    fn fix_bowling_allies() {
        assert_suggestion_result(
            "Bowling Allies,Casinos,Gyms,Movie Rentals,Movie Theaters,Museums",
            AlleyAlly::default(),
            "Bowling Alleys,Casinos,Gyms,Movie Rentals,Movie Theaters,Museums",
        );
    }

    #[test]
    fn dont_flag_damp_ally() {
        assert_no_lints(
            "Thankfully we have a damp ally with a penchant for chaos himself",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn dont_flag_dank_allies() {
        assert_no_lints(
            "Remind me again- which nationalities count as the \"Dank Allies\"?",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn fix_dark_ally() {
        assert_suggestion_result(
            "eating rats for sustenance in a dark ally somewhere",
            AlleyAlly::default(),
            "eating rats for sustenance in a dark alley somewhere",
        );
    }

    #[test]
    fn fix_dark_allies() {
        assert_suggestion_result(
            "because them dark allies are a pain in the ass to navigate",
            AlleyAlly::default(),
            "because them dark alleys are a pain in the ass to navigate",
        );
    }

    #[test]
    fn fix_darkest_ally() {
        assert_suggestion_result(
            "You live in the scariest, darkest ally, with a big mansion",
            AlleyAlly::default(),
            "You live in the scariest, darkest alley, with a big mansion",
        );
    }

    #[test]
    fn fix_dingy_ally() {
        assert_suggestion_result(
            "But it definitely was surprising to be walking down a dingy ally at 2 am and seeing a lone 20yo woman just playing on her phone",
            AlleyAlly::default(),
            "But it definitely was surprising to be walking down a dingy alley at 2 am and seeing a lone 20yo woman just playing on her phone",
        );
    }

    #[test]
    fn dont_flag_hidden_ally() {
        assert_no_lints(
            "For in its quiet persistence, boredom may prove less an enemy than a hidden ally",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn dont_flag_sketchy_allies() {
        assert_no_lints(
            "usually polite even to his enemies (or sketchy allies, like the Queens and Bronx groups)",
            AlleyAlly::default(),
        );
    }

    // "ally" <word> with words that are normally used with "alley"

    #[test]
    fn fix_ally_cat() {
        assert_suggestion_result(
            "Ally cat comes by every morning to say hello",
            AlleyAlly::default(),
            "Alley cat comes by every morning to say hello",
        );
    }

    #[test]
    fn fix_ally_cats() {
        assert_suggestion_result(
            "I have two cats at my actual home who were former ally cats & adopted us",
            AlleyAlly::default(),
            "I have two cats at my actual home who were former alley cats & adopted us",
        );
    }

    #[test]
    fn fix_ally_oop_hyphen() {
        assert_suggestion_result(
            "for the ally-oop and additional details on timeline/availability",
            AlleyAlly::default(),
            "for the alley-oop and additional details on timeline/availability",
        );
    }

    #[test]
    fn fix_drone_ally_oops() {
        assert_suggestion_result(
            "3 successful drone ally oops per period - then stop",
            AlleyAlly::default(),
            "3 successful drone alley oops per period - then stop",
        );
    }

    #[test]
    fn fix_ally_way() {
        assert_suggestion_result(
            "Alex agreed and they entered the ally way outside the gloomy studio.",
            AlleyAlly::default(),
            "Alex agreed and they entered the alley way outside the gloomy studio.",
        );
    }

    #[test]
    fn fix_ally_ways_hyphen() {
        assert_suggestion_result(
            "chairs that people left in the backstreets and ally-ways in Hong Kong and China",
            AlleyAlly::default(),
            "chairs that people left in the backstreets and alley-ways in Hong Kong and China",
        );
    }

    #[test]
    fn fix_ally_ways_space() {
        assert_suggestion_result(
            "tourists such as myself, scuffling along paved ally ways",
            AlleyAlly::default(),
            "tourists such as myself, scuffling along paved alley ways",
        );
    }

    // <adjective> "alley" with adjectives normally used with "ally".
    // Ruled out after being found before both:
    // closest alleys, European alley(s), natural alleys, strong alley, traditional alley(s)

    #[test]
    fn dont_flag_closest_alley() {
        assert_no_lints(
            "However the closest alley to me is ten pin only",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn dont_flag_closest_alleys() {
        assert_no_lints(
            "the 8 closest alleys all charge for lanes by the hour",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn dont_flag_key_alley() {
        assert_no_lints(
            "You can place it further away but pointed and zoomed at a key alley/street etc.",
            AlleyAlly::default(),
        );
    }

    #[test]
    #[ignore = "also occurs legitimately"]
    fn fix_key_alley() {
        assert_suggestion_result(
            "Both Israel and its key alley, the United States, have been working together.",
            AlleyAlly::default(),
            "Both Israel and its key ally, the United States, have been working together.",
        );
    }

    #[test]
    fn dont_flag_key_alley_title_case() {
        assert_no_lints(
            "What is this thing on Key Alley? It had doors on the inside garden side but they look sealed shut.",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn dont_flag_top_of_the_key_alley_oops() {
        assert_no_lints(
            "I've never seen so many top-of-the-key alley oops from a single player since Lonzo",
            AlleyAlly::default(),
        );
    }

    #[test]
    #[ignore = "also occurs legitimately"]
    fn fix_longtime_alley() {
        assert_suggestion_result(
            "And the Avengers would know that their longtime alley isn't himself after he breaks his ex-gf jaw.",
            AlleyAlly::default(),
            "And the Avengers would know that their longtime ally isn't himself after he breaks his ex-gf jaw.",
        );
    }

    #[test]
    fn dont_flag_longtime_alley() {
        assert_no_lints(
            "plans for a carriage house collide with a longtime alley garden",
            AlleyAlly::default(),
        );
    }

    #[test]
    #[ignore = "also occurs legitimately"]
    fn fix_natural_alley() {
        assert_suggestion_result(
            "They have a natural alley, that helps them to pursue their special interests using force - the state.",
            AlleyAlly::default(),
            "They have a natural ally, that helps them to pursue their special interests using force - the state.",
        );
    }

    #[test]
    fn dont_flag_natural_alley() {
        assert_no_lints(
            "The natural alley between the two major buildings makes for a very easily walled off area",
            AlleyAlly::default(),
        );
    }

    #[test]
    #[ignore = "also occurs legitimately"]
    fn fix_political_alleys() {
        assert_suggestion_result(
            "family that want to have more money and castle, and more political alleys",
            AlleyAlly::default(),
            "family that want to have more money and castle, and more political allies",
        );
    }

    #[test]
    fn dont_flag_dark_political_alleys() {
        assert_no_lints(
            "what is currently hot in the dark political alleys",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn fix_potential_alley() {
        assert_suggestion_result(
            "effectively making an entire enemy type into potential alleys",
            AlleyAlly::default(),
            "effectively making an entire enemy type into potential allies",
        );
    }

    #[test]
    fn dont_flag_potential_alley_oop() {
        assert_no_lints(
            "which result in a cut and a potential alley oop",
            AlleyAlly::default(),
        );
    }

    #[test]
    #[ignore = "also occurs legitimately"]
    fn fix_powerful_alley() {
        assert_suggestion_result(
            "Make it a powerful alley for those seeking to expand their holdings.",
            AlleyAlly::default(),
            "Make it a powerful ally for those seeking to expand their holdings.",
        );
    }

    #[test]
    fn dont_flag_powerful_alley_oop_hyphen() {
        assert_no_lints(
            "LeBron James with the powerful alley-oop dunk off a lob",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn fix_reliable_alley() {
        assert_suggestion_result(
            "Many organizations now use AI as a reliable alley in their exercises.",
            AlleyAlly::default(),
            "Many organizations now use AI as a reliable ally in their exercises.",
        );
    }

    #[test]
    fn dont_flag_strong_alley_title_case() {
        assert_no_lints(
            "The owners of the Strong Alley buildings have agreed to allow family-appropriate(ish) mural art",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn fix_strongest_alley() {
        assert_suggestion_result(
            "Patience will be your strongest alley in this journey.",
            AlleyAlly::default(),
            "Patience will be your strongest ally in this journey.",
        );
    }

    #[test]
    fn dont_flag_tough_alley_oop() {
        assert_no_lints(
            "Victor Wembanyama misses the tough alley-oop dunk",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn fix_useful_alley() {
        assert_suggestion_result(
            "Even if Obito saw Itachi as a useful alley and what not it still makes no sense",
            AlleyAlly::default(),
            "Even if Obito saw Itachi as a useful ally and what not it still makes no sense",
        );
    }

    #[test]
    fn dont_flag_useless_alleys() {
        assert_no_lints(
            "Towns, at least early game, are filled with useless alleys and halls",
            AlleyAlly::default(),
        );
    }

    // "ally oneself" misspelled as "alley oneself"
    // turns out to have many false positives, so we're not flagging it

    #[test]
    fn dont_flag_alley_herself() {
        assert_no_lints(
            "Gloria Rayman tackled Dounton Alley herself.",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn dont_flag_alley_herself_with() {
        assert_no_lints(
            "to oppose her mother she must alley herself with what terrifies herself most",
            AlleyAlly::default(),
        );
    }

    #[test]
    #[ignore = "also occurs legitimately"]
    fn fix_alley_himself_with_union() {
        assert_suggestion_result(
            "chose rather to alley himself with the pagan tribal union",
            AlleyAlly::default(),
            "chose rather to ally himself with the pagan tribal union",
        );
    }

    #[test]
    #[ignore = "also occurs legitimately"]
    fn fix_alley_himself_with_pandora() {
        assert_suggestion_result(
            "he's clearly not going to join or alley himself with Pandora",
            AlleyAlly::default(),
            "he's clearly not going to join or ally himself with Pandora",
        );
    }

    #[test]
    fn dont_flag_alley_itself_to() {
        assert_no_lints(
            "The design of the alley and surrounding spaces is focused on allowing the alley itself to become a comfortable pedestrian environment",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn dont_flag_alley_itself_with() {
        assert_no_lints(
            "increasing pedestrian traffic and establishing the alley itself with a sense of place",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn dont_flag_alleys_myself() {
        assert_no_lints(
            "I go down those streets, visit those towns and alleys myself sometimes STILL!",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn dont_flag_alley_myself_to() {
        assert_no_lints(
            "Once I single-handedly shoveled half the alley myself to get out.",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn dont_flag_alley_myself_with() {
        assert_no_lints(
            "I went down the alley myself with gloves and picked up the trash",
            AlleyAlly::default(),
        );
    }

    #[test]
    #[ignore = "also occurs legitimately"]
    fn fix_alley_ourselves_with() {
        assert_suggestion_result(
            "We have come to alley ourselves with you, sons of Muspellheim",
            AlleyAlly::default(),
            "We have come to ally ourselves with you, sons of Muspellheim",
        );
    }

    #[test]
    fn dont_flag_alley_themselves() {
        assert_no_lints(
            "So effectively they end up dead in an alley themselves.",
            AlleyAlly::default(),
        );
    }

    #[test]
    #[ignore = "also occurs legitimately"]
    fn fix_alley_themselves_with_strahd() {
        assert_suggestion_result(
            "desperate souls seeking to alley themselves with Strahd",
            AlleyAlly::default(),
            "desperate souls seeking to ally themselves with Strahd",
        );
    }

    #[test]
    #[ignore = "also occurs legitimately"]
    fn fix_alley_themselves_with_decrees() {
        assert_suggestion_result(
            "They don't alley themselves with the king's decrees",
            AlleyAlly::default(),
            "They don't ally themselves with the king's decrees",
        );
    }

    #[test]
    #[ignore = "also occurs legitimately"]
    fn fix_alley_themselves_with_superheroes() {
        assert_suggestion_result(
            "The koalas alley themselves with the rest of the superheroes in the series",
            AlleyAlly::default(),
            "The koalas ally themselves with the rest of the superheroes in the series",
        );
    }

    #[test]
    fn dont_flag_alley_yourself() {
        assert_no_lints(
            "you have to find a parking spot in the alley yourself",
            AlleyAlly::default(),
        );
    }

    #[test]
    fn dont_flag_alleys_yourself_to() {
        assert_no_lints(
            "You have to wander your way through the maze of side streets and alleys yourself to understand how it feels to be there.",
            AlleyAlly::default(),
        );
    }
}
