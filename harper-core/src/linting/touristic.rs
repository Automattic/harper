use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenStringExt,
    expr::{Expr, LongestMatchOf, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SuggestionPreference {
    /// Explicitly allow this suggestion
    Allow,
    /// Explicitly deny this suggestion
    Deny,
    /// Not explicitly allowed or denied
    #[default]
    Neutral,
}

use SuggestionPreference::*;

pub struct Touristic {
    expr: Box<dyn crate::expr::Expr>,
}

// "touristy" doesn't sound natural with these words
const BLACKLIST: &[&str] = &[
    "app",
    "apps",
    "data",
    "content",
    "establishment",
    "establishments",
    "info",
    "information",
    "interest",
    "platform",
    "platforms",
    "service",
    "services",
];

// "touristy" sounds natural with these words
const WHITELIST: &[&str] = &[
    "activity",
    "activities",
    "area",
    "areas",
    "destination",
    "destinations",
    "location",
    "locations",
    "place",
    "places",
    "route",
    "routes",
    "spot",
    "spots",
];

impl Default for Touristic {
    fn default() -> Self {
        let with_prev_and_next_word = SequenceExpr::any_word()
            .t_ws()
            .t_aco("touristic")
            .t_ws()
            .then_any_word();

        let with_prev_word = SequenceExpr::any_word().t_ws().t_aco("touristic");

        let with_next_word = SequenceExpr::default()
            .t_aco("touristic")
            .t_ws()
            .then_any_word();

        let pattern = LongestMatchOf::new(vec![
            Box::new(with_prev_and_next_word),
            Box::new(with_prev_word),
            Box::new(with_next_word),
            Box::new(SequenceExpr::default().t_aco("touristic")),
        ]);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for Touristic {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tok_span_content_string = toks.span()?.get_content_string(src);
        let tok_span_content_string = tok_span_content_string.to_lowercase();

        let mut touristy_pref = Neutral;
        let mut noun_forms_pref = Neutral;

        let span_number = match (
            toks.len(),
            tok_span_content_string.starts_with("touristic "),
            tok_span_content_string.ends_with(" touristic"),
        ) {
            (1, _, _) => {
                noun_forms_pref = Allow;
                touristy_pref = Allow;
                0
            }
            (3, true, false) => {
                let next_word = toks[2].span.get_content_string(src);
                let next_kind = &toks[2].kind;

                if next_kind.is_noun() {
                    if WHITELIST.contains(&next_word.as_str()) {
                        touristy_pref = Allow;
                    }
                    if BLACKLIST.contains(&next_word.as_str()) {
                        touristy_pref = Deny;
                    }
                }
                0
            }
            (3, false, true) => {
                let prev_kind = &toks[0].kind;
                noun_forms_pref = if prev_kind.is_adjective() || prev_kind.is_linking_verb() {
                    Deny
                } else {
                    Allow
                };
                2
            }
            (5, _, _) => {
                let _prev_word = toks[0].span.get_content_string(src).to_lowercase();
                let prev_kind = &toks[0].kind;
                let next_word = toks[4].span.get_content_string(src).to_lowercase();
                let next_kind = &toks[4].kind;

                if prev_kind.is_adverb() {
                    noun_forms_pref = Deny;
                }

                if next_kind.is_noun() {
                    if WHITELIST.contains(&next_word.as_str()) {
                        touristy_pref = Allow;
                    }
                    if BLACKLIST.contains(&next_word.as_str()) {
                        touristy_pref = Deny;
                    }
                }

                if next_kind.is_adjective() && !next_kind.is_noun() {
                    noun_forms_pref = Deny;
                    touristy_pref = Allow;
                }
                2
            }
            _ => return None,
        };

        let mut suggested = Vec::new();
        if noun_forms_pref != Deny {
            suggested.push("tourist");
            suggested.push("tourism");
        }
        if touristy_pref != Deny {
            suggested.push("touristy");
        }

        let span = toks[span_number].span;
        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: suggested
                .into_iter()
                .map(|s| Suggestion::replace_with_match_case_str(s, span.get_content(src)))
                .collect(),
            message: "The word `touristic` is rarely used by native speakers.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Suggests replacing the uncommon word `touristic` with `tourist`, `tourism`, and/or `touristy`."
    }
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
    fn fixes_t_data() {
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
