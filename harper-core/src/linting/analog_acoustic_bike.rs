use crate::{
    Dialect, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::IndefiniteArticle,
};

pub struct AnalogAcousticBike {
    expr: SequenceExpr,
    dialect: Dialect,
}

impl AnalogAcousticBike {
    pub fn new(dialect: Dialect) -> Self {
        let expr = SequenceExpr::optional(SequenceExpr::with(IndefiniteArticle::default()).t_ws())
            .t_set(&["acoustic", "analog", "analogue"])
            .t_ws()
            .t_set(&["bike", "bikes", "bicycle", "bicycles"]);

        Self { expr, dialect }
    }
}

impl ExprLinter for AnalogAcousticBike {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // TODO: address the optional indefinite article
        let toks = &toks[toks.len() - 3..];

        let full_span = toks.span()?;

        let ws_cycle = toks[1..=2].get_ch(src)?;
        let cycle = toks[2..].get_ch(src)?;

        let join = |qualifier: &str, cycle: &[char]| -> Vec<char> {
            qualifier.chars().chain(cycle.iter().copied()).collect()
        };

        let is_bike = cycle.contains(&'k');
        let use_push = matches!(self.dialect, Dialect::British | Dialect::Australian);

        Some(Lint {
            span: full_span,
            lint_kind: LintKind::WordChoice,
            suggestions: [
                (use_push, "push", if is_bike { cycle } else { ws_cycle }),
                (true, "pedal", ws_cycle),
            ]
            .into_iter()
            .filter(|(add_ws, _, _)| *add_ws)
            .map(|(_, qualifier, cycle)| {
                Suggestion::replace_with_match_case(
                    join(qualifier, cycle),
                    full_span.get_content(src),
                )
            })
            .collect(),
            message: "Consider a more standard term.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Suggests more standard terms for `analog/analogue bike` and `acoustic bike`."
    }
}

#[cfg(test)]
mod tests {
    use crate::{Dialect, linting::tests::assert_good_and_bad_suggestions};

    use super::AnalogAcousticBike;

    #[test]
    fn fix_acoustic_bike_us() {
        assert_good_and_bad_suggestions(
            "you might get something that helps you move around your city faster and with less sweat than an acoustic bike",
            AnalogAcousticBike::new(Dialect::American),
            &[
                "you might get something that helps you move around your city faster and with less sweat than an pedal bike",
            ],
            &[
                "you might get something that helps you move around your city faster and with less sweat than an pushbike",
            ],
        );
    }

    #[test]
    fn fix_acoustic_bikes_uk() {
        assert_good_and_bad_suggestions(
            "Subjectively, it feels as adequately braked as any of my acoustic bikes",
            AnalogAcousticBike::new(Dialect::British),
            &[
                "Subjectively, it feels as adequately braked as any of my pedal bikes",
                "Subjectively, it feels as adequately braked as any of my pushbikes",
            ],
            &[],
        );
    }

    #[test]
    fn fix_analog_bike_ca() {
        assert_good_and_bad_suggestions(
            "Pretty sure almost anyone can go faster downhill on an analog bike than the 25kph that Ebike pedal assistance is limited to.",
            AnalogAcousticBike::new(Dialect::Canadian),
            &[
                "Pretty sure almost anyone can go faster downhill on an pedal bike than the 25kph that Ebike pedal assistance is limited to.",
            ],
            &[
                "Pretty sure almost anyone can go faster downhill on an pushbike than the 25kph that Ebike pedal assistance is limited to.",
            ],
        );
    }

    #[test]
    fn fix_analog_bikes_au() {
        assert_good_and_bad_suggestions(
            "These bikes are not modified analog bikes — they are ground-up designs built for assisted momentum.",
            AnalogAcousticBike::new(Dialect::Australian),
            &[
                "These bikes are not modified pedal bikes — they are ground-up designs built for assisted momentum.",
                "These bikes are not modified pushbikes — they are ground-up designs built for assisted momentum.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_acoustic_bicycle_uk() {
        assert_good_and_bad_suggestions(
            "I still play alcohol powered instruments (double bass, cello), and ride an acoustic bicycle",
            AnalogAcousticBike::new(Dialect::British),
            &[
                "I still play alcohol powered instruments (double bass, cello), and ride an pedal bicycle",
            ],
            &[
                "I still play alcohol powered instruments (double bass, cello), and ride an pushbike",
            ],
        );
    }

    #[test]
    fn fix_analog_bicycles_us() {
        assert_good_and_bad_suggestions(
            "both light electric bicycles and heavy analog bicycles exist",
            AnalogAcousticBike::new(Dialect::American),
            &["both light electric bicycles and heavy pedal bicycles exist"],
            &["both light electric bicycles and heavy pushbikes exist"],
        );
    }

    #[test]
    fn fix_analogue_bicycle_au() {
        assert_good_and_bad_suggestions(
            "There's crossover with a good ol' analogue bicycle, but I found it awkward to rock up at places and events sweating.",
            AnalogAcousticBike::new(Dialect::Australian),
            &[
                "There's crossover with a good ol' pedal bicycle, but I found it awkward to rock up at places and events sweating.",
                "There's crossover with a good ol' push bicycle, but I found it awkward to rock up at places and events sweating.",
            ],
            &[],
        );
    }
}
