use crate::{
    Token, TokenStringExt,
    linting::{Lint, LintKind, PatternLinter, Suggestion},
    patterns::{EitherPattern, SequencePattern},
};

pub struct Touristic {
    pattern: Box<dyn crate::patterns::Pattern>,
}

const TOURISTY_NOUN_BLACKLIST: &[&str] = &[
    "app",
    "data",
    "content",
    "establishment",
    "info",
    "information",
    "interest",
    "platform",
    "service",
    "establishments",
    "platforms",
    "services",
];
const TOURISTY_NOUN_WHITELIST: &[&str] = &[
    "activity",
    "area",
    "destination",
    "location",
    "place",
    "route",
    "spot",
    "activities",
    "areas",
    "destinations",
    "locations",
    "places",
    "routes",
    "spots",
];

impl Default for Touristic {
    fn default() -> Self {
        let with_prev_and_next_word = SequencePattern::default()
            .then_any_word()
            .t_ws()
            .t_aco("touristic")
            .t_ws()
            .then_any_word();

        let with_prev_word = SequencePattern::default()
            .then_any_word()
            .t_ws()
            .t_aco("touristic");

        let with_next_word = SequencePattern::default()
            .t_aco("touristic")
            .t_ws()
            .then_any_word();

        let pattern = EitherPattern::new(vec![
            Box::new(with_prev_and_next_word),
            Box::new(with_prev_word),
            Box::new(with_next_word),
            Box::new(SequencePattern::default().t_aco("touristic")),
        ]);

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for Touristic {
    fn pattern(&self) -> &dyn crate::patterns::Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tok_span_content_string = toks.span()?.get_content_string(src);
        eprintln!("⭐️ {}", tok_span_content_string); //toks.span()?.get_content_string(src));
        let tok_span_content_string = tok_span_content_string.to_lowercase();
        match toks.len() {
            1 => {
                eprintln!("❤️ match_to_lint got 1 token = 1 word");
                return Some(Lint {
                    span: toks[0].span,
                    lint_kind: LintKind::WordChoice,
                    suggestions: vec![
                        Suggestion::replace_with_match_case_str(
                            "tourist",
                            toks[0].span.get_content(src),
                        ),
                        Suggestion::replace_with_match_case_str(
                            "tourism",
                            toks[0].span.get_content(src),
                        ),
                        Suggestion::replace_with_match_case_str(
                            "touristy",
                            toks[0].span.get_content(src),
                        ),
                    ],
                    message: "The word \"touristic\" is rarely used by native speakers."
                        .to_string(),
                    priority: 31,
                });
            }
            3 => {
                eprintln!("🍏🍏 match_to_lint got 3 tokens = 2 words");
                if tok_span_content_string.starts_with("touristic ") {
                    eprintln!("🍏🍏 got 3 tokens = 2 words, starting with \"touristic \"");
                    
                    let next_word = toks[2].span.get_content_string(src);
                    let next_kind = &toks[2].kind;
                    let pos_string = tok_pos(next_kind);
                    eprintln!("🍏🍏 \"touristic\" \"{next_word}\"<{pos_string}>🍏🍏");

                    let mut can_suggest_touristy = None;
                    let mut can_suggest_tourist_and_tourism: Option<bool> = None;

                    if next_kind.is_noun() {
                        if TOURISTY_NOUN_WHITELIST.contains(&next_word.as_str()) {
                            eprintln!("🍏<🤍> 'touristy {}' sounds fine", next_word);
                            if can_suggest_touristy == Some(false) {
                                eprintln!("🖤 noun white list overriding touristy suggestion");
                            }
                            can_suggest_touristy = Some(true);
                        }
                        if TOURISTY_NOUN_BLACKLIST.contains(&next_word.as_str()) {
                            eprintln!("🍏<🖤> 'touristy {}' sounds weird", next_word);
                            if can_suggest_touristy == Some(true) {
                                eprintln!("🖤 noun black list overriding touristy suggestion");
                            }
                            can_suggest_touristy = Some(false);
                        }
                    }

                    let mut suggestions = vec![];
                    if can_suggest_tourist_and_tourism == Some(true) || can_suggest_tourist_and_tourism == None {
                        eprintln!("🍏 can suggest tourist+tourism: {:?}", can_suggest_tourist_and_tourism);
                        suggestions.push(Suggestion::replace_with_match_case_str(
                            "tourist",
                            toks[0].span.get_content(src),
                        ));
                        suggestions.push(Suggestion::replace_with_match_case_str(
                            "tourism",
                            toks[0].span.get_content(src),
                        ));
                    }
                    if can_suggest_touristy == Some(true) || can_suggest_touristy == None {
                        eprintln!("🍏 can suggest touristy: {:?}", can_suggest_touristy);
                        suggestions.push(Suggestion::replace_with_match_case_str(
                            "touristy",
                            toks[0].span.get_content(src),
                        ));
                    }
                    
                    return Some(Lint {
                        span: toks[0].span,
                        lint_kind: LintKind::WordChoice,
                        suggestions,
                        message: "The word \"touristic\" is rarely used by native speakers."
                            .to_string(),
                        priority: 31,
                    });
                }
                if !tok_span_content_string.ends_with(" touristic") {
                    eprintln!(
                        "❤️ ❤️ got 3 tokens = 2 words but it doesn't start or end with \"touristic\""
                    );
                    return None;
                }

                let prev_word = toks[0].span.get_content_string(src);
                let prev_kind = &toks[0].kind;
                let pos_string = tok_pos(prev_kind);
                eprintln!("❤️ ❤️ \"{prev_word}\"<{pos_string}> \"touristic\" ❤️ ❤️");
                let suggestions = if prev_kind.is_adjective() || prev_kind.is_linking_verb() {
                    vec![Suggestion::replace_with_match_case_str(
                        "touristy",
                        toks[0].span.get_content(src),
                    )]
                } else {
                    vec![
                        Suggestion::replace_with_match_case_str(
                            "tourist",
                            toks[0].span.get_content(src),
                        ),
                        Suggestion::replace_with_match_case_str(
                            "tourism",
                            toks[0].span.get_content(src),
                        ),
                        Suggestion::replace_with_match_case_str(
                            "touristy",
                            toks[0].span.get_content(src),
                        ),
                    ]
                };
                return Some(Lint {
                    span: toks[2].span,
                    lint_kind: LintKind::WordChoice,
                    suggestions,
                    message: "The word \"touristic\" is rarely used by native speakers."
                        .to_string(),
                    priority: 31,
                });
            }
            5 => {
                eprintln!("❤️ ❤️ ❤️ match_to_lint got 5 tokens = 3 words");
                let prev_word = toks[0].span.get_content_string(src);
                let prev_kind = &toks[0].kind;
                let next_word = toks[4].span.get_content_string(src);
                let next_kind = &toks[4].kind;
                let prev_pos_string = tok_pos(prev_kind);
                let next_pos_string = tok_pos(next_kind);
                eprintln!(
                    "❤️ ❤️ ❤️ \"{prev_word}\"<{prev_pos_string}> \"touristic\" \"{next_word}\"<{next_pos_string}>"
                );
                let (prev_word, next_word) = (prev_word.to_lowercase(), next_word.to_lowercase());
                eprintln!(
                    "🍒🍒🍒 \"{prev_word}\"<{prev_pos_string}> \"touristic\" \"{next_word}\"<{next_pos_string}>"
                );

                let mut can_suggest_touristy = None;
                let mut can_suggest_tourist_and_tourism = None;

                // only add "tourist" and "tourism" if prev word is not adverb
                if prev_kind.is_adverb() {
                    eprintln!("🍊<🖤> '{} tourist' sounds weird", prev_word);
                    if can_suggest_tourist_and_tourism == Some(true) {
                        eprintln!("🖤 adverb rule overriding tourist/tourism suggestion");
                    }
                    can_suggest_tourist_and_tourism = Some(false);
                }

                if next_kind.is_noun() {
                    if TOURISTY_NOUN_WHITELIST.contains(&next_word.as_str()) {
                        eprintln!("🍓<🤍> 'touristy {}' sounds fine", next_word);
                        if can_suggest_touristy == Some(false) {
                            eprintln!("🖤 noun white list overriding touristy suggestion");
                        }
                        can_suggest_touristy = Some(true);
                    }
                    if TOURISTY_NOUN_BLACKLIST.contains(&next_word.as_str()) {
                        eprintln!("🍓<🖤> 'touristy {}' sounds weird", next_word);
                        if can_suggest_touristy == Some(true) {
                            eprintln!("🖤 noun black list overriding touristy suggestion");
                        }
                        can_suggest_touristy = Some(false);
                    }
                }

                if next_kind.is_adjective() && !next_kind.is_noun() {
                    if can_suggest_tourist_and_tourism == Some(true) {
                        eprintln!("🖤 adj rule overriding tourist/tourism suggestion");
                    }
                    can_suggest_tourist_and_tourism = Some(false);
                    if can_suggest_touristy == Some(false) {
                        eprintln!("🖤 adj rule overriding touristy suggestion");
                    }
                    can_suggest_touristy = Some(true);
                }

                let mut suggestions = vec![];
                match can_suggest_tourist_and_tourism {
                    Some(true) | None => {
                        suggestions.push(Suggestion::replace_with_match_case_str(
                            "tourist",
                            toks[2].span.get_content(src),
                        ));
                        suggestions.push(Suggestion::replace_with_match_case_str(
                            "tourism",
                            toks[2].span.get_content(src),
                        ));
                    }
                    Some(false) => {}
                }
                match can_suggest_touristy {
                    Some(true) | None => {
                        suggestions.push(Suggestion::replace_with_match_case_str(
                            "touristy",
                            toks[2].span.get_content(src),
                        ));
                    }
                    Some(false) => {} //| None => {}
                }

                return Some(Lint {
                    span: toks[2].span,
                    lint_kind: LintKind::WordChoice,
                    suggestions,
                    message: "The word \"touristic\" is rarely used by native speakers."
                        .to_string(),
                    priority: 31,
                });
            }
            _ => {
                eprintln!(
                    "❤️💎❤️ match_to_lint got an unexpected {} tokens",
                    toks.len()
                );
            }
        }
        None
    }

    fn description(&self) -> &'static str {
        "Suggests replacing the uncommon word \"touristic\" with \"tourist\", \"tourism\", or \"touristy\"."
    }
}

fn tok_pos(tok_kind: &crate::TokenKind) -> String {
    let mut next_pos = vec![];
    if tok_kind.is_noun() {
        next_pos.push("n.");
    }
    if tok_kind.is_linking_verb() {
        next_pos.push("v.link.");
    } else if tok_kind.is_verb() {
        next_pos.push("v.");
    }
    if tok_kind.is_adjective() {
        next_pos.push("adj.");
    }
    if tok_kind.is_adverb() {
        next_pos.push("adv.");
    }
    if tok_kind.is_pronoun() {
        next_pos.push("pron.");
    }
    if tok_kind.is_conjunction() {
        next_pos.push("conj.");
    }
    if tok_kind.is_preposition() {
        next_pos.push("prep.");
    }
    let res = if next_pos.is_empty() {
        "??".to_string()
    } else {
        next_pos.join(" ").to_string()
    };
    res
}

#[cfg(test)]
mod tests {
    use super::Touristic;
    use crate::linting::tests::assert_good_and_bad_suggestions;

    #[test]
    fn fixes_touristic_alone() {
        assert_good_and_bad_suggestions(
            "touristic",
            Touristic::default(),
            &["tourist", "tourism", "touristy"],
            &[],
        );
    }

    #[test]
    fn fixes_very_t() {
        assert_good_and_bad_suggestions(
            "very touristic",
            Touristic::default(),
            &["very touristy"],
            &["very tourist", "very tourism"],
        );
    }

    #[test]
    fn fixes_t_location_good_and_bad() {
        assert_good_and_bad_suggestions(
            "touristic location",
            Touristic::default(),
            &["tourist location", "tourism location", "touristy location"],
            &[],
        );
    }

    #[test]
    fn fixes_is_t() {
        assert_good_and_bad_suggestions(
            "That place is touristic",
            Touristic::default(),
            &["That place is touristy"],
            &["That place is tourist", "That place is tourism"],
        );
    }

    #[test]
    fn fixes_t_information() {
        assert_good_and_bad_suggestions(
            "The AI Touristic Information Tool for Liquid Galaxy is a Flutter-based Android tablet application that simplifies and enhances travel planning.",
            Touristic::default(),
            &[
                "The AI Tourist Information Tool for Liquid Galaxy is a Flutter-based Android tablet application that simplifies and enhances travel planning.",
                "The AI Tourism Information Tool for Liquid Galaxy is a Flutter-based Android tablet application that simplifies and enhances travel planning.",
            ],
            &[
                "The AI Touristy Information Tool for Liquid Galaxy is a Flutter-based Android tablet application that simplifies and enhances travel planning.",
            ],
        );
    }

    #[test]
    fn fices_t_data() {
        assert_good_and_bad_suggestions(
            "Official API to access Apidae touristic data.",
            Touristic::default(),
            &[
                "Official API to access Apidae tourist data.",
                "Official API to access Apidae tourism data.",
            ],
            &["Official API to access Apidae touristy data."],
        );
    }

    #[test]
    fn corrects_t_information_2() {
        assert_good_and_bad_suggestions(
            "Oppidums is open source app that provide cultural, historical and touristic information on different cities.",
            Touristic::default(),
            &[
                "Oppidums is open source app that provide cultural, historical and tourist information on different cities.",
                "Oppidums is open source app that provide cultural, historical and tourism information on different cities.",
            ],
            &[
                "Oppidums is open source app that provide cultural, historical and touristy information on different cities.",
            ],
        );
    }

    #[test]
    fn corrects_very_t_spot() {
        assert_good_and_bad_suggestions(
            "The destination is a very touristic spot, many people visit this place at the weekend.",
            Touristic::default(),
            &[
                "The destination is a very touristy spot, many people visit this place at the weekend.",
            ],
            &[
                "The destination is a very tourist spot, many people visit this place at the weekend.",
                "The destination is a very tourism spot, many people visit this place at the weekend.",
            ],
        );
    }

    #[test]
    #[ignore = "Checks previous word but results depend on the next word"]
    fn fixes_t_platform() {
        assert_good_and_bad_suggestions(
            "Incuti is touristic platform for African destinations.",
            Touristic::default(),
            &[
                "Incuti is tourist platform for African destinations.",
                "Incuti is tourism platform for African destinations.",
            ],
            &["Incuti is touristy platform for African destinations."],
        );
    }

    #[test]
    fn fixes_t_service_providers() {
        assert_good_and_bad_suggestions(
            "Onlim API is a tool that touristic service providers utilize to generate social media posts by injecting data about their offers into some templates.",
            Touristic::default(),
            &[
                "Onlim API is a tool that tourist service providers utilize to generate social media posts by injecting data about their offers into some templates.",
                "Onlim API is a tool that tourism service providers utilize to generate social media posts by injecting data about their offers into some templates.",
            ],
            &[
                "Onlim API is a tool that touristy service providers utilize to generate social media posts by injecting data about their offers into some templates.",
            ],
        );
    }

    #[test]
    fn fixes_are_t_areas() {
        assert_good_and_bad_suggestions(
            "We can determine that most of the busier areas are touristic areas, which in return helps with the high demand for the shared bikes.",
            Touristic::default(),
            &[
                "We can determine that most of the busier areas are tourist areas, which in return helps with the high demand for the shared bikes.",
                "We can determine that most of the busier areas are tourism areas, which in return helps with the high demand for the shared bikes.",
                "We can determine that most of the busier areas are touristy areas, which in return helps with the high demand for the shared bikes.",
            ],
            &[],
        );
    }

    #[test]
    fn fixes_very_t_area() {
        assert_good_and_bad_suggestions(
            "This is Manhattan, a very popular, very touristic area of New York.",
            Touristic::default(),
            &["This is Manhattan, a very popular, very touristy area of New York."],
            &[
                "This is Manhattan, a very popular, very tourist area of New York.",
                "This is Manhattan, a very popular, very tourism area of New York.",
            ],
        );
    }

    #[test]
    fn fixes_for_t_photographic() {
        assert_good_and_bad_suggestions(
            "Python implementation of my clustering-based recommendation system for touristic photographic spots.",
            Touristic::default(),
            &[
                "Python implementation of my clustering-based recommendation system for touristy photographic spots.",
            ],
            &[
                "Python implementation of my clustering-based recommendation system for tourist photographic spots.",
                "Python implementation of my clustering-based recommendation system for tourism photographic spots.",
            ],
        );
    }

    #[test]
    fn fixes_czech_t_routes() {
        assert_good_and_bad_suggestions(
            "Management and Control Application for Czech Touristic Routes in OSM.",
            Touristic::default(),
            &[
                "Management and Control Application for Czech Tourist Routes in OSM.",
                "Management and Control Application for Czech Tourism Routes in OSM.",
                "Management and Control Application for Czech Touristy Routes in OSM.",
            ],
            &[],
        );
    }

    #[test]
    fn fixes_promote_t_activities() {
        assert_good_and_bad_suggestions(
            "Application to promote touristic activities in Valencia.",
            Touristic::default(),
            &[
                "Application to promote tourist activities in Valencia.",
                "Application to promote tourism activities in Valencia.",
                "Application to promote touristy activities in Valencia.",
            ],
            &[],
        );
    }

    #[test]
    fn fixes_for_t_content() {
        assert_good_and_bad_suggestions(
            "Missing languages for published field in APIv2 for Touristic Content",
            Touristic::default(),
            &[
                "Missing languages for published field in APIv2 for Tourist Content",
                "Missing languages for published field in APIv2 for Tourism Content",
            ],
            &["Missing languages for published field in APIv2 for Touristy Content"],
        );
    }

    #[test]
    fn fixes_a_t_flutter() {
        assert_good_and_bad_suggestions(
            "A Touristic Flutter App.",
            Touristic::default(),
            &["A Tourist Flutter App.", "A Tourism Flutter App."],
            &[
                // "app" would be fine in the blacklist, but "Flutter" would be going too far
                // "A Touristy Flutter App.",
            ],
        );
    }

    #[test]
    fn fixes_of_t_interest() {
        assert_good_and_bad_suggestions(
            "ARCHEO: a python lib for sound event detection in areas of touristic Interest.",
            Touristic::default(),
            &[
                "ARCHEO: a python lib for sound event detection in areas of tourist Interest.",
                "ARCHEO: a python lib for sound event detection in areas of tourism Interest.",
            ],
            &["ARCHEO: a python lib for sound event detection in areas of touristy Interest."],
        );
    }

    #[test]
    fn fixes_t_establishments() {
        assert_good_and_bad_suggestions(
            "Touristic establishments by EUROSTAT NUTS regions.",
            Touristic::default(),
            &[
                "Tourist establishments by EUROSTAT NUTS regions.",
                "Tourism establishments by EUROSTAT NUTS regions.",
            ],
            &["Touristy establishments by EUROSTAT NUTS regions."],
        );
    }
}
