use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        debug::format_lint_match,
        expr_linter::{Chunk, find_the_only_token_index_matching},
    },
};

const ALL_GAUGE: [&str; 8] = [
    "gauge", "guage", "gauged", "guauged", "gauges", "guauges", "gauging", "guauging",
];

const ALL_GOUGE: [&str; 4] = ["gouge", "gouged", "gouges", "gouging"];

const BEFORE_GAUGE: &[&str] = &[
    // Measurement, Engineering & Physics (Strictly gauge):
    // Before gauge(s/ing)
    "depth",
    "fuel",
    "ionization",
    "level",
    "pressure",
    "radiation",
    "rain",
    "river",
    "strain",
    "stream",
    "tank",
    "temperature",
    "thickness",
    "tide",
    "tire",
    "tyre",
    "vacuum",
    "wire",
    // Before gauge(d) (Rail/Scale)
    "broad",
    "lattice",
    "metre",
    "narrow",
    "standard",
    "unified",
];

const BEFORE_GOUGE: &[&str] = &[
    "price",
    "rent",
    // Physical Carving, Geology & Metalworking (Strictly gouge):
    // Before gouge(s/ing)
    "air",
    "arc",
    "chisel",
    "chisels",
    "clay",
    "crescentic",
    "crusher",
    "deep",
    "fault",
    "flame",
    "ice",
    // Violence & Idioms (Strictly gouge):
    // Before/After gouge(d/ing)
    "eye",
    "eyes",
    "out", // (gouge out, gouged out).
];

const AFTER_GAUGE: &[&str] = &[
    // After gauge(s/ing)
    "apparatus",
    "boson",
    "field",
    "fields",
    "instruments",
    "invariance",
    "needle",
    "railroad",
    "railway",
    "station",
    "stations",
    "theories",
    "theory",
];

const AFTER_GOUGE: &[&str] = &[
    // After gouge(s/ing):
    "abrasion", "machines", "marks", "nozzles", "torch", "zone", // (geological fault gouge),
    // Violence & Idioms (Strictly gouge):
    // Before/After gouge(d/ing)
    "eye", "eyes", "out", // (gouge out, gouged out).
];

pub struct GaugeGouge {
    expr: SequenceExpr,
}

impl Default for GaugeGouge {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_of([
                Box::new(
                    SequenceExpr::default()
                        .t_set(ALL_GAUGE)
                        .t_ws()
                        .t_set(AFTER_GOUGE),
                ),
                Box::new(
                    SequenceExpr::default()
                        .t_set(BEFORE_GOUGE)
                        .t_ws()
                        .t_set(ALL_GAUGE),
                ),
                Box::new(
                    SequenceExpr::default()
                        .t_set(ALL_GOUGE)
                        .t_ws()
                        .t_set(AFTER_GAUGE),
                ),
                Box::new(
                    SequenceExpr::default()
                        .t_set(BEFORE_GAUGE)
                        .t_ws()
                        .t_set(ALL_GOUGE),
                ),
            ]),
        }
    }
}

impl ExprLinter for GaugeGouge {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));
        let Some(the_only) = find_the_only_token_index_matching(toks, src, |t, s| {
            let ch = t.get_ch(s);
            ch.eq_any_ignore_ascii_case_str(&ALL_GAUGE)
                || ch.eq_any_ignore_ascii_case_str(&ALL_GOUGE)
        }) else {
            return None;
        };

        let span = toks.get(the_only)?.span;

        let mut word = span.get_content(src).to_vec();

        // We need to flip one vowel when either word is correctly spelled.
        // But we need to flip two vowels when 'guage' is misspelled.
        if word.starts_with_any_ignore_ascii_case_str(&["gau", "gou"]) {
            let v = word.get_mut(1)?;

            *v = ((*v as u8) ^ (b'a' ^ b'o')) as char;
        } else {
            let [v1, v2] = word.get_disjoint_mut([1, 2]).ok()?;

            *v1 = ((*v1 as u8) ^ (b'u' ^ b'o')) as char;
            *v2 = ((*v2 as u8) ^ (b'a' ^ b'u')) as char;
        }

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith(word)],
            message: "Have you confused `gauge` (measure) with `gouge` (exploit)?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Looks for `gauge` and `gouge` mixed up."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::GaugeGouge;

    #[test]
    fn price_gauging() {
        assert_suggestion_result(
            "Price gauging has a very particular meaning; it isn't simply a subjective \"you're charging too much\".",
            GaugeGouge::default(),
            "Price gouging has a very particular meaning; it isn't simply a subjective \"you're charging too much\".",
        );
    }

    #[test]
    fn fuel() {
        assert_suggestion_result(
            "look at the fuel gouge",
            GaugeGouge::default(),
            "look at the fuel gauge",
        );
    }

    #[test]
    fn gauge() {
        assert_suggestion_result(
            "there's a deep guage in the tank",
            GaugeGouge::default(),
            "there's a deep gouge in the tank",
        );
    }
}
