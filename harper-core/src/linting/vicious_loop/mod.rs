use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

#[derive(PartialEq)]
enum Prefer {
    Circle,
    Cycle,
    DontCare,
}

pub struct ViciousCircle {
    expr: Box<dyn Expr>,
}
pub struct ViciousCycle {
    expr: Box<dyn Expr>,
}
pub struct ViciousCircleOrCycle {
    expr: Box<dyn Expr>,
}

// The Expr must have all three tokens because they should only be flagged when used together.
// But we don't want to flag the legitimate combinations, and which those are depends on the user's preferences.
fn build_expr(flag: Prefer) -> Box<dyn Expr> {
    let seq = SequenceExpr::word_set(&["vicious", "virtuous", "viscous"])
        .t_ws()
        .then_word_set(&["circle", "circles", "cycle", "cycles"]);

    match flag {
        Prefer::Circle => Box::new(
            seq.and_not(
                SequenceExpr::default()
                    .then_word_except(&["viscous"])
                    .t_ws()
                    .then_word_set(&["circle", "circles"]),
            ),
        ),
        Prefer::Cycle => Box::new(
            seq.and_not(
                SequenceExpr::default()
                    .then_word_except(&["viscous"])
                    .t_ws()
                    .then_word_set(&["cycle", "cycles"]),
            ),
        ),
        Prefer::DontCare => {
            Box::new(seq.and_not(SequenceExpr::default().then_word_except(&["viscous"])))
        }
    }
}

fn to_lint(toks: &[Token], src: &[char], pref: Prefer) -> Option<Lint> {
    let tokspan = toks.span()?;
    let (adjtok, nountok) = (toks.first()?, toks.last()?);

    let badadj = adjtok
        .span
        .get_content(src)
        .eq_ignore_ascii_case_chars(&['v', 'i', 's', 'c', 'o', 'u', 's']);

    let badnoun = match pref {
        Prefer::Circle => nountok
            .span
            .get_content(src)
            .starts_with_ignore_ascii_case_str("cycle"),
        Prefer::Cycle => nountok
            .span
            .get_content(src)
            .starts_with_ignore_ascii_case_str("circle"),
        Prefer::DontCare => false,
    };

    let is_plural = matches!(nountok.span.get_content(src).last(), Some('s' | 'S'));

    // The noun doesn't match the user's preferred word.
    if badnoun && !badadj {
        return Some(Lint {
            span: nountok.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                match (&pref, is_plural) {
                    (Prefer::Circle, false) => "circle",
                    (Prefer::Circle, true) => "circles",
                    (Prefer::Cycle, false) => "cycle",
                    (Prefer::Cycle, true) => "cycles",
                    _ => unreachable!(),
                },
                nountok.span.get_content(src),
            )],
            message: if pref == Prefer::Circle {
                "This idiom originally used `circle`, not `cycle`".to_string()
            } else {
                "Though this idiom originally used `circle`, `cycle` is preferred.".to_string()
            },
            ..Default::default()
        });
    }

    // The noun doesn't match the user's preference *and* the adjective also needs to be corrected from "viscous" to "vicious"
    if badnoun && badadj {
        let nouns = &["circle", "cycle"];
        let i = match pref {
            Prefer::Circle => 0,
            Prefer::Cycle => 1,
            Prefer::DontCare => return None, // Unreachable, but we don't risk crashing the LSP.
        };

        let message = format!(
            "The idiom uses the word `vicious`, not `viscous`, which describes thick liquids. And we prefer `{}` over `{}`.",
            nouns[i],
            nouns[1 - i],
        );

        return Some(Lint {
            span: tokspan,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                match (&pref, is_plural) {
                    (Prefer::Circle, false) => "vicious circle",
                    (Prefer::Circle, true) => "vicious circles",
                    (Prefer::Cycle, false) => "vicious cycle",
                    (Prefer::Cycle, true) => "vicious cycles",
                    _ => return None, // Unreachable, but we don't risk crashing the LSP.
                },
                tokspan.get_content(src),
            )],
            message,
            ..Default::default()
        });
    }

    // Nouns are fine, but we need to correct "viscous" to "vicious".
    if badadj {
        return Some(Lint {
            span: adjtok.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "vicious",
                adjtok.span.get_content(src),
            )],
            message:
                "The idiom uses the word `vicious`, not `viscous`, which describes thick liquids."
                    .to_string(),
            ..Default::default()
        });
    }

    None
}

macro_rules! impl_expr_linter {
    ($name:ident, $pref:expr, $desc:expr) => {
        impl Default for $name {
            fn default() -> Self {
                Self {
                    expr: build_expr($pref),
                }
            }
        }

        impl ExprLinter for $name {
            type Unit = Chunk;

            fn description(&self) -> &str {
                $desc
            }

            fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
                to_lint(toks, src, $pref)
            }

            fn expr(&self) -> &dyn Expr {
                self.expr.as_ref()
            }
        }
    };
}

impl_expr_linter!(
    ViciousCircle,
    Prefer::Circle,
    "Corrects and standardizes common errors and variants of `vicious/virtuous circle`."
);

impl_expr_linter!(
    ViciousCycle,
    Prefer::Cycle,
    "Corrects and standardizes common errors and variants of `vicious/virtuous cycle`."
);

impl_expr_linter!(
    ViciousCircleOrCycle,
    Prefer::DontCare,
    "Corrects common errors in `vicious/virtuous circle/cycle`."
);

#[cfg(test)]
mod tests {
    use super::{ViciousCircle, ViciousCircleOrCycle, ViciousCycle};
    use crate::linting::tests::{
        assert_no_lints, assert_suggestion_result, assert_top3_suggestion_result,
    };

    // Prefer "circle" -  Made up, simple examples

    #[test]
    fn vicious_singular() {
        assert_suggestion_result("vicious cycle", ViciousCircle::default(), "vicious circle");
    }
    #[test]
    fn vicious_plural() {
        assert_suggestion_result(
            "vicious cycles",
            ViciousCircle::default(),
            "vicious circles",
        );
    }
    #[test]
    fn viscous_singular() {
        assert_suggestion_result("viscous cycle", ViciousCircle::default(), "vicious circle");
    }
    #[test]
    fn viscous_plural() {
        assert_suggestion_result(
            "viscous cycles",
            ViciousCircle::default(),
            "vicious circles",
        );
    }

    #[test]
    fn ignore_vicious_singular() {
        assert_no_lints("vicious circle", ViciousCircle::default());
    }
    #[test]
    fn ignore_virtuous_plural() {
        assert_no_lints("virtuous circles", ViciousCircle::default());
    }

    // Prefer "circle" -  Real-world examples

    #[test]
    fn fix_singular_and_plural_nouns() {
        assert_top3_suggestion_result(
            "The file Vicious Cycle Dataset.ods contains 33 vicious cycles from 13 open source systems studied in our paper.",
            ViciousCircle::default(),
            "The file Vicious Circle Dataset.ods contains 33 vicious circles from 13 open source systems studied in our paper.",
        );
    }

    #[test]
    fn fix_virtuous() {
        assert_suggestion_result(
            "FlashInfer-Bench is a benchmark suite and production workflow designed to build a virtuous cycle of self-improving AI systems.",
            ViciousCircle::default(),
            "FlashInfer-Bench is a benchmark suite and production workflow designed to build a virtuous circle of self-improving AI systems.",
        );
    }

    // Prefer "cycle" - Made up, simple examples

    #[test]
    fn fix_singular() {
        assert_suggestion_result("vicious circle", ViciousCycle::default(), "vicious cycle");
    }
    #[test]
    fn fix_plural() {
        assert_suggestion_result(
            "virtuous circles",
            ViciousCycle::default(),
            "virtuous cycles",
        );
    }
    #[test]
    fn fix_viscous_singular() {
        assert_suggestion_result("viscous circle", ViciousCycle::default(), "vicious cycle");
    }
    #[test]
    fn fix_viscous_plural() {
        assert_suggestion_result("viscous circles", ViciousCycle::default(), "vicious cycles");
    }
    #[test]
    fn dont_flag_singular() {
        assert_no_lints("viscious cycle", ViciousCycle::default());
    }
    #[test]
    fn dont_flag_plural() {
        assert_no_lints("virtuous cycles", ViciousCycle::default());
    }

    // Prefer "cycle" -  Real-world examples

    #[test]
    fn fix_its_a_virtuous() {
        assert_suggestion_result(
            "It's a virtuous circle: if it's interesting to do a project, a person spends a lot of time on it",
            ViciousCycle::default(),
            "It's a virtuous cycle: if it's interesting to do a project, a person spends a lot of time on it",
        );
    }

    #[test]
    #[ignore = "Harper currently misinterprets the words around the ellipses as a hostname"]
    fn fix_viscous() {
        assert_suggestion_result(
            "However, adding it to $connectionsToTransact causes the tests to stop running...viscous circle.",
            ViciousCycle::default(),
            "However, adding it to $connectionsToTransact causes the tests to stop running...vicious cycle.",
        );
    }

    // No preference - both "circle" and "cycle" are fine.

    #[test]
    fn dont_flag_either() {
        assert_no_lints(
            "vicious circle, virtuous cycle, vicious cycles, virtuous circles",
            ViciousCircleOrCycle::default(),
        );
    }

    #[test]
    fn fix_both_viscous() {
        assert_suggestion_result(
            "viscous circle, viscous cycles",
            ViciousCircleOrCycle::default(),
            "vicious circle, vicious cycles",
        );
    }

    // No preference - Real-world examples

    #[test]
    fn dont_flag_combo() {
        assert_no_lints(
            "Instead of a vicious cycle, popularity creates a virtuous circle.",
            ViciousCircleOrCycle::default(),
        );
    }

    #[test]
    fn fix_its_a_viscous_cycle() {
        assert_suggestion_result(
            "Its a viscous cycle that started back in 1.13 for a few plugins but is now hurting every single world generation plugin",
            ViciousCircleOrCycle::default(),
            "Its a vicious cycle that started back in 1.13 for a few plugins but is now hurting every single world generation plugin",
        );
    }
}
