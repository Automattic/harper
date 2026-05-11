use crate::{
    Lint, Token, TokenStringExt,
    char_string::CharStringExt,
    expr::{Expr, FirstMatchOf, OwnedExprExt, ReflexivePronoun, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct RedundantSelf {
    expr: FirstMatchOf,
}

impl Default for RedundantSelf {
    fn default() -> Self {
        // Pattern 1: Reflexive Verb Pattern
        // self-harm myself, self-teach yourself, etc.
        let reflexive_verb = Box::new(
            SequenceExpr::aco("self")
                .t_ws_h()
                .t_set(&[
                    "harm", "harmed", "harming", "harms", "taught", "teach", "teaches", "teaching",
                ])
                .t_ws()
                .then(ReflexivePronoun::with_common_errors())
                .and_not(
                    SequenceExpr::anything()
                        .t_any()
                        .t_aco("harm")
                        .t_any()
                        .t_aco("itself"),
                ),
        );

        // Pattern 2: Noun Pattern
        // self-harm to myself, self-harm to yourself, etc.
        let noun = Box::new(
            SequenceExpr::aco("self")
                .t_ws_h()
                .t_aco("harm")
                .t_ws()
                .t_aco("to")
                .t_ws()
                .then(ReflexivePronoun::with_common_errors()),
        );

        // Pattern 3: Transitive Verb Pattern
        // self-host it themselves, self-host one yourself, etc.
        let transitive_verb = Box::new(
            SequenceExpr::aco("self")
                .t_ws_h()
                .t_set(&["host", "hosted", "hosting", "hosts"])
                .t_ws()
                .t_set(&["it", "them", "one"])
                .t_ws()
                .then(ReflexivePronoun::with_common_errors()),
        );

        Self {
            expr: FirstMatchOf::new(vec![reflexive_verb, noun, transitive_verb]),
        }
    }
}

impl ExprLinter for RedundantSelf {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.span()?;

        let suggestions = match (toks.len(), toks[4].get_ch(src).eq_ch(&['t', 'o'])) {
            // Reflexive Verb Pattern: self-harm myself, self-teach yourself
            (5, _) => [0..=2, 2..=4],
            // Noun Pattern: self-harm to myself, self-harm to yourself
            (7, true) => [0..=2, 2..=6],
            // Transitive Verb Pattern: self-host it themselves, self-host one yourself
            (7, false) => [0..=4, 2..=6],
            _ => {
                return None;
            }
        }
        .iter()
        .map(|range| range.clone().collect::<Vec<_>>())
        .map(|indices| {
            Suggestion::replace_with_match_case(
                indices
                    .into_iter()
                    .flat_map(|i| toks[i].get_ch(src))
                    .copied()
                    .collect(),
                span.get_content(src),
            )
        })
        .collect();

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Redundancy,
            suggestions,
            message: "Avoid redundancy by using `self` only once.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects redundant use of `self-` prefixes with reflexive pronouns (e.g., `self-host it themselves`)."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
    };

    use super::RedundantSelf;

    // Self-harm onself test

    #[test]
    fn fix_self_harm_herself_hy() {
        assert_good_and_bad_suggestions(
            "Camille about to self-harm herself",
            RedundantSelf::default(),
            &[
                "Camille about to harm herself",
                "Camille about to self-harm",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_harming_herself_hy() {
        assert_good_and_bad_suggestions(
            "At times she was suicidal and started self-harming herself when she was 12.",
            RedundantSelf::default(),
            &[
                "At times she was suicidal and started harming herself when she was 12.",
                "At times she was suicidal and started self-harming when she was 12.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_harming_herself_sp() {
        assert_good_and_bad_suggestions(
            "Violet self harming herself is inter-spliced in this montage and Tate walks in to tell her she's cutting the wrong way.",
            RedundantSelf::default(),
            &[
                "Violet harming herself is inter-spliced in this montage and Tate walks in to tell her she's cutting the wrong way.",
                "Violet self harming is inter-spliced in this montage and Tate walks in to tell her she's cutting the wrong way.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_harming_himself_hy() {
        assert_good_and_bad_suggestions(
            "I don't know the reason till now for doing so but I'm sure he was self-harming himself.",
            RedundantSelf::default(),
            &[
                "I don't know the reason till now for doing so but I'm sure he was harming himself.",
                "I don't know the reason till now for doing so but I'm sure he was self-harming.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_harming_myself_sp() {
        assert_good_and_bad_suggestions(
            "I'm no programmer or anything like that but I like staying up late at night and self harming myself trying to get ajax to work.",
            RedundantSelf::default(),
            &[
                "I'm no programmer or anything like that but I like staying up late at night and harming myself trying to get ajax to work.",
                "I'm no programmer or anything like that but I like staying up late at night and self harming trying to get ajax to work.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_harming_myself_hy() {
        assert_good_and_bad_suggestions(
            "How I stopped self-harming myself",
            RedundantSelf::default(),
            &["How I stopped harming myself", "How I stopped self-harming"],
            &[],
        );
    }

    #[test]
    fn fix_self_harm_to_myself_hy() {
        assert_good_and_bad_suggestions(
            "I'm glad that I didn't do any self-harm to myself",
            RedundantSelf::default(),
            &[
                "I'm glad that I didn't do any harm to myself",
                "I'm glad that I didn't do any self-harm",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_harm_to_myself_sp() {
        assert_good_and_bad_suggestions(
            "so I did self harm to myself with the teacher noticing",
            RedundantSelf::default(),
            &[
                "so I did harm to myself with the teacher noticing",
                "so I did self harm with the teacher noticing",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_harmed_myself_sp() {
        assert_good_and_bad_suggestions(
            "I haven't self harmed myself in exactly 3 months and 4 days!!!",
            RedundantSelf::default(),
            &[
                "I haven't harmed myself in exactly 3 months and 4 days!!!",
                "I haven't self harmed in exactly 3 months and 4 days!!!",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_harmed_themself_sp() {
        assert_good_and_bad_suggestions(
            "trusted you enough to tell you that they self harmed themself",
            RedundantSelf::default(),
            &[
                "trusted you enough to tell you that they harmed themself",
                "trusted you enough to tell you that they self harmed",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_harming_yourself_hy() {
        assert_good_and_bad_suggestions(
            "Is psychologically self-harming yourself on purpose possible",
            RedundantSelf::default(),
            &[
                "Is psychologically harming yourself on purpose possible",
                "Is psychologically self-harming on purpose possible",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_harming_yourself_sp() {
        assert_good_and_bad_suggestions(
            "What isn't seen as self harming yourself but actually is?",
            RedundantSelf::default(),
            &[
                "What isn't seen as harming yourself but actually is?",
                "What isn't seen as self harming but actually is?",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_harm_to_yourself_hy() {
        assert_good_and_bad_suggestions(
            "First of all, do not turn to physical abuse or any type of self-harm to yourself.",
            RedundantSelf::default(),
            &[
                "First of all, do not turn to physical abuse or any type of harm to yourself.",
                "First of all, do not turn to physical abuse or any type of self-harm.",
            ],
            &[],
        );
    }

    // Self-teaching onself tests

    #[test]
    fn fix_self_teaching_myself() {
        assert_suggestion_result(
            "This is a reference of me self-teaching myself the Barvinok algorithm.",
            RedundantSelf::default(),
            "This is a reference of me teaching myself the Barvinok algorithm.",
        );
    }

    #[test]
    fn fix_self_taught_myself() {
        assert_suggestion_result(
            "Since I wasn't blessed to learn security related topic back when I was undergrad, I self taught myself using udemy, and CTF Challenge online.",
            RedundantSelf::default(),
            "Since I wasn't blessed to learn security related topic back when I was undergrad, I taught myself using udemy, and CTF Challenge online.",
        );
    }

    #[test]
    fn fix_self_teach_myself() {
        assert_suggestion_result(
            "I decided to self-teach myself python so I could get more involved in wider aspects of research, specifically data science.",
            RedundantSelf::default(),
            "I decided to teach myself python so I could get more involved in wider aspects of research, specifically data science.",
        );
    }

    // The point of OSSU is to make things easier for people self-teaching themselves CS.
    #[test]
    fn fix_self_teaching_themselves() {
        assert_suggestion_result(
            "The point of OSSU is to make things easier for people self-teaching themselves CS.",
            RedundantSelf::default(),
            "The point of OSSU is to make things easier for people teaching themselves CS.",
        );
    }

    // Self-host X onself pattern tests

    #[test]
    fn fix_self_host_it_yourself() {
        assert_good_and_bad_suggestions(
            "If you want to self-host it yourself, get the latest release and download the distributable zip file attached to it",
            RedundantSelf::default(),
            &[
                "If you want to self-host it, get the latest release and download the distributable zip file attached to it",
                "If you want to host it yourself, get the latest release and download the distributable zip file attached to it",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_hosting_it_themselves() {
        assert_good_and_bad_suggestions(
            "I've open-sourced the codebase and written an installation guide to make it as easy as possible for others who are interested in self-hosting it themselves.",
            RedundantSelf::default(),
            &[
                "I've open-sourced the codebase and written an installation guide to make it as easy as possible for others who are interested in self-hosting it.",
                "I've open-sourced the codebase and written an installation guide to make it as easy as possible for others who are interested in hosting it themselves.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_self_hosted_one_myself() {
        assert_good_and_bad_suggestions(
            "I though WalletConnect bridge server might be too flooded with requests, so I self-hosted one myself, same result.",
            RedundantSelf::default(),
            &[
                "I though WalletConnect bridge server might be too flooded with requests, so I self-hosted one, same result.",
                "I though WalletConnect bridge server might be too flooded with requests, so I hosted one myself, same result.",
            ],
            &[],
        );
    }

    // Avoiding false positives

    #[test]
    fn dont_flag_self_harm_itself() {
        assert_no_lints(
            "I understand that self harm itself isn't something that you would have to report.",
            RedundantSelf::default(),
        );
    }

    #[test]
    fn dont_flag_self_host_without_pronoun() {
        assert_no_lints(
            "I prefer to self-host my applications",
            RedundantSelf::default(),
        );
    }

    #[test]
    fn dont_flag_host_without_self() {
        assert_no_lints(
            "They decided to host it themselves",
            RedundantSelf::default(),
        );
    }
}
