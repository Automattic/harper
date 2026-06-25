use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, OwnedExprExt, ReflexivePronoun, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct SelfHarmOneself {
    expr: FirstMatchOf,
}

impl Default for SelfHarmOneself {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::aco("self")
                        .t_ws_h()
                        .t_set(&["harm", "harmed", "harming", "harms"])
                        .t_ws()
                        .then(ReflexivePronoun::with_common_errors())
                        .but_not(
                            SequenceExpr::anything()
                                .t_any()
                                .t_aco("harm")
                                .t_any()
                                .t_aco("itself"),
                        ),
                ) as Box<dyn Expr>,
                Box::new(
                    SequenceExpr::aco("self")
                        .t_ws_h()
                        .t_aco("harm")
                        .t_ws()
                        .t_aco("to")
                        .t_ws()
                        .then(ReflexivePronoun::with_common_errors()),
                ),
            ]),
        }
    }
}

impl ExprLinter for SelfHarmOneself {
    type Unit = Chunk;

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = tokens.span()?;

        let suggestions = match tokens.len() {
            5 => [0..=2, 2..=4],
            7 => [0..=2, 2..=6],
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
                    .flat_map(|i| tokens[i].get_ch(source))
                    .copied()
                    .collect(),
                span.get_content(source),
            )
        })
        .collect();

        Some(Lint {
            span: tokens.span()?,
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
        "A linter skeleton for contributors to copy into `harper_core/src/linting/` and rename."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_no_lints};

    use super::SelfHarmOneself;

    // Basic funcitonality test

    #[test]
    fn fix_verb() {
        assert_good_and_bad_suggestions(
            "Don't self-harm yourself",
            SelfHarmOneself::default(),
            &["Don't harm yourself", "Don't self-harm"],
            &[],
        );
    }

    #[test]
    fn fix_noun() {
        assert_good_and_bad_suggestions(
            "Don't do self-harm to yourself",
            SelfHarmOneself::default(),
            &["Don't do harm to yourself", "Don't do self-harm"],
            &[],
        );
    }

    // Real-world test cases

    #[test]
    fn fix_self_harm_herself_hy() {
        assert_good_and_bad_suggestions(
            "Camille about to self-harm herself",
            SelfHarmOneself::default(),
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
            SelfHarmOneself::default(),
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
            SelfHarmOneself::default(),
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
            SelfHarmOneself::default(),
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
            SelfHarmOneself::default(),
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
            SelfHarmOneself::default(),
            &["How I stopped harming myself", "How I stopped self-harming"],
            &[],
        );
    }

    #[test]
    fn fix_self_harm_to_myself_hy() {
        assert_good_and_bad_suggestions(
            "I'm glad that I didn't do any self-harm to myself",
            SelfHarmOneself::default(),
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
            SelfHarmOneself::default(),
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
            SelfHarmOneself::default(),
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
            SelfHarmOneself::default(),
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
            SelfHarmOneself::default(),
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
            SelfHarmOneself::default(),
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
            SelfHarmOneself::default(),
            &[
                "First of all, do not turn to physical abuse or any type of harm to yourself.",
                "First of all, do not turn to physical abuse or any type of self-harm.",
            ],
            &[],
        );
    }

    // Avoiding false positives

    #[test]
    fn dont_flag_self_harm_itself() {
        assert_no_lints(
            "I understand that self harm itself isn't something that you would have to report.",
            SelfHarmOneself::default(),
        );
    }
}
