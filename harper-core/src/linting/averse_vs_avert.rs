use crate::{
    CharStringExt, Lint, Token, TokenKind,
    expr::{AnchorStart, Expr, FirstMatchOf, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
    patterns::{ContractionOfBe, InflectionOfBe},
};

const AVERSE: &[&str] = &["averse", "aversed", "averses", "aversing"];

pub struct AverseVsAvert {
    expr: FirstMatchOf,
}

impl Default for AverseVsAvert {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::any_of([
                        Box::new(
                            SequenceExpr::any_of([
                                Box::new(ContractionOfBe::new()) as Box<dyn Expr>,
                                Box::new(InflectionOfBe::new()),
                                Box::new(SequenceExpr::default().then_kind_either(
                                    TokenKind::is_quantifier,
                                    TokenKind::is_degree_adverb,
                                )),
                            ])
                            .t_ws(),
                        ) as Box<dyn Expr>,
                        Box::new(AnchorStart),
                    ])
                    .then_any_of([
                        Box::new(SequenceExpr::word_seq(&["avert", "to"])) as Box<dyn Expr>,
                        Box::new(SequenceExpr::aco("risk").t_ws_h().t_aco("averted")),
                    ]),
                ),
                Box::new(
                    SequenceExpr::word_set(AVERSE)
                        .t_ws()
                        .then_possessive_determiner()
                        .t_ws()
                        .t_aco("eyes"),
                ),
                Box::new(
                    SequenceExpr::word_set(&["risk", "conflict"])
                        .t_ws_h()
                        .t_aco("avert"),
                ),
            ]),
        }
    }
}

impl ExprLinter for AverseVsAvert {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tok = find_the_only_token_matching(toks, src, |t, s| {
            let ch = t.get_ch(s);
            ch.eq_any_ignore_ascii_case_str(AVERSE)
                || ch.eq_any_ignore_ascii_case_str(&["avert", "averted"])
        })?;

        let ch = tok.get_ch(src);

        let correction = match (ch.get(4)?, ch.last()?) {
            ('s', 'e') => "avert",
            ('s', 'd') => "averted",
            ('s', 's') => "averts",
            ('s', 'g') => "averting",
            ('t', 't' | 'd') => "averse",
            _ => return None,
        };

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Malapropism,
            suggestions: vec![Suggestion::replace_with_match_case_str(correction, ch)],
            message: "Did you mean `averse` (strongly opposed to) rather than `avert` (to prevent or turn away)?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects confusing `avert` with `averse`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AverseVsAvert;

    // ... be avert to ... -> be averse to ...
    // ... very avert to ... -> very averse to ...
    // Avert to ...

    #[test]
    fn avert_to_at_start() {
        assert_suggestion_result(
            "Avert to the spotlight.",
            AverseVsAvert::default(),
            "Averse to the spotlight.",
        );
    }

    #[test]
    fn are_avert_to() {
        assert_suggestion_result(
            "People want the game to evolve, but are avert to change.",
            AverseVsAvert::default(),
            "People want the game to evolve, but are averse to change.",
        );
    }

    #[test]
    fn be_avert_to() {
        assert_suggestion_result(
            "It's like how in our world most humans will be avert to kill or torture creatures of the in-group",
            AverseVsAvert::default(),
            "It's like how in our world most humans will be averse to kill or torture creatures of the in-group",
        );
    }

    #[test]
    fn being_avert_to() {
        assert_suggestion_result(
            "rather than being avert to challenges and turbulence, we must embrace it",
            AverseVsAvert::default(),
            "rather than being averse to challenges and turbulence, we must embrace it",
        );
    }

    #[test]
    fn being_avert_to_x2() {
        assert_suggestion_result(
            "Its not people being avert to long stories its people being avert to stories that can end without a long period of time",
            AverseVsAvert::default(),
            "Its not people being averse to long stories its people being averse to stories that can end without a long period of time",
        );
    }

    #[test]
    fn been_avert_to() {
        assert_suggestion_result(
            "My grandmother (94 yo) has recently been avert to eating.",
            AverseVsAvert::default(),
            "My grandmother (94 yo) has recently been averse to eating.",
        );
    }

    #[test]
    fn dont_flag_to_avert_to() {
        assert_no_lints(
            "leaders met throughout the night to find a way to avert today,s protest, but were unsuccessful",
            AverseVsAvert::default(),
        );
    }

    // risk/conflict-avert -> risk/conflict-averse
    // be risk/conflict averted -> be risk/conflict-averse

    #[test]
    fn am_risk_avert() {
        assert_suggestion_result(
            "I would not say that I am risk avert maybe but rather risk conscious.",
            AverseVsAvert::default(),
            "I would not say that I am risk averse maybe but rather risk conscious.",
        );
    }

    #[test]
    fn are_risk_avert() {
        assert_suggestion_result(
            "households and communities are risk avert and face significant barriers to adopting livelihood strategies",
            AverseVsAvert::default(),
            "households and communities are risk averse and face significant barriers to adopting livelihood strategies",
        );
    }

    #[test]
    fn being_risk_avert() {
        assert_suggestion_result(
            "the Bank’s attempt to quickly respond to the country’s immediate transport challenges, and not being risk avert, is commendable",
            AverseVsAvert::default(),
            "the Bank’s attempt to quickly respond to the country’s immediate transport challenges, and not being risk averse, is commendable",
        );
    }

    #[test]
    fn youre_conflict_avert() {
        assert_suggestion_result(
            "Knowing you're conflict-avert doesn't excuse hiding you true opinions.",
            AverseVsAvert::default(),
            "Knowing you're conflict-averse doesn't excuse hiding you true opinions.",
        );
    }

    #[test]
    fn less_risk_avert() {
        assert_suggestion_result(
            "studies have shown that atheists are actually less risk avert than theists",
            AverseVsAvert::default(),
            "studies have shown that atheists are actually less risk averse than theists",
        );
    }

    #[test]
    fn more_risk_avert() {
        assert_suggestion_result(
            "to be cautious and like you said becoming more risk avert",
            AverseVsAvert::default(),
            "to be cautious and like you said becoming more risk averse",
        );
    }

    #[test]
    fn too_risk_avert() {
        assert_suggestion_result(
            "people not too risk-avert with no one depending on them, or people with a safety net",
            AverseVsAvert::default(),
            "people not too risk-averse with no one depending on them, or people with a safety net",
        );
    }

    #[test]
    fn very_risk_averted() {
        assert_suggestion_result(
            "Most people are just very risk averted.",
            AverseVsAvert::default(),
            "Most people are just very risk averse.",
        );
    }

    #[test]
    fn risk_averted_not_at_start_or_after_be() {
        assert_no_lints(
            "Status: passed; one critical risk averted (per design-review F-2)",
            AverseVsAvert::default(),
        );
    }

    #[test]
    fn risk_avert_title_case_hyphenated() {
        assert_suggestion_result(
            "if anyone's children have been offered to take part in a programme called Risk-Avert in school",
            AverseVsAvert::default(),
            "if anyone's children have been offered to take part in a programme called Risk-Averse in school",
        );
    }

    // averse one's eyes -> avert one's eyes

    #[test]
    fn averse_his_eyes() {
        assert_suggestion_result(
            "His claim that he would never averse his eyes from slaughter has proved, sadly, quite hollow.",
            AverseVsAvert::default(),
            "His claim that he would never avert his eyes from slaughter has proved, sadly, quite hollow.",
        );
    }

    #[test]
    fn averse_her_eyes() {
        assert_suggestion_result(
            "Ei's apathy and decision to isolate herself and averse her eyes from her own people suffering is called out",
            AverseVsAvert::default(),
            "Ei's apathy and decision to isolate herself and avert her eyes from her own people suffering is called out",
        );
    }

    // False positives to avoid

    #[test]
    fn dont_flag_risk_averters() {
        assert_no_lints(
            "Some dynasties of agents are risk averters, and others are risk lovers.",
            AverseVsAvert::default(),
        );
    }

    // Not handled but found in the wild

    // Malloy plants a sustained kiss on Howard's shocked lips in a scene which
    //   might have caused a 1950s audience to avert to their eyes.
}
