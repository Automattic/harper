use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

const TOTE: &[&str] = &["tote", "toted", "totes", "toting"];
const TOUT: &[&str] = &["tout", "touted", "touting", "touts"];

const WEIRD_BEFORE_TOTE: &[&str] = &["ads", "highly", "much", "often", "ticket", "widely"];
const WEIRD_BEFORE_TOUT: &[&str] = &["canvas", "gun", "pistol", "plastic"];
const WEIRD_AFTER_TOTE: &[&str] = &["as"];
const WEIRD_AFTER_TOUT: &[&str] = &["bag", "bags", "boxes"];

pub struct ToteTout {
    expr: FirstMatchOf,
}

impl Default for ToteTout {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::word_set(WEIRD_BEFORE_TOTE)
                        .t_ws_h()
                        .t_set(TOTE),
                ),
                Box::new(
                    SequenceExpr::word_set(WEIRD_BEFORE_TOUT)
                        .t_ws_h()
                        .t_set(TOUT),
                ),
                Box::new(SequenceExpr::word_set(TOTE).t_ws().t_set(WEIRD_AFTER_TOTE)),
                Box::new(SequenceExpr::word_set(TOUT).t_ws().t_set(WEIRD_AFTER_TOUT)),
            ]),
        }
    }
}

impl ExprLinter for ToteTout {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = find_the_only_token_matching(toks, src, |t, _| {
            let ch = t.get_ch(src);
            ch.eq_any_ignore_ascii_case_str(TOTE) || ch.eq_any_ignore_ascii_case_str(TOUT)
        })?
        .span;
        let ch = span.get_content(src);

        // This is the place to rule out collocations with specific inflected forms, if need be.

        // Map from wrong word to right word.
        let (third, last) = (ch.get(2)?, ch.last()?);
        let corr = match (third, last) {
            ('u', 't') => "tote",
            ('u', 'd') => "toted",
            ('u', 'g') => "toting",
            ('u', 's') => "totes",
            ('t', 'e') => "tout",
            ('t', 'd') => "touted",
            ('t', 's') => "touts",
            ('t', 'g') => "touting",
            _ => return None,
        };

        let suggestions = vec![Suggestion::replace_with_match_case_str(
            corr,
            span.get_content(src),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message: "Are you confusng `tote` (carry, bag) with `tout` (promote, promoter)?"
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags places where `tote` and `tout` may be confused."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ToteTout;

    #[test]
    fn fix_toted_as_a_tool() {
        assert_suggestion_result(
            "remember cloudformation is toted as a tool for resource creation, not orchestration",
            ToteTout::default(),
            "remember cloudformation is touted as a tool for resource creation, not orchestration",
        );
    }

    #[test]
    fn fix_often_toted_as_the_worst() {
        assert_suggestion_result(
            "He is often toted as the worst killer in history",
            ToteTout::default(),
            "He is often touted as the worst killer in history",
        )
    }

    #[test]
    fn fix_gun_touting_space() {
        assert_suggestion_result(
            "Batman doesn't use guns now is because his back-story includes a gun touting coward stealing away his parents",
            ToteTout::default(),
            "Batman doesn't use guns now is because his back-story includes a gun toting coward stealing away his parents",
        )
    }

    #[test]
    fn fix_much_toted_space() {
        assert_suggestion_result(
            "I realize it's much toted for having a brilliant API, but unfortunately that's not sufficient",
            ToteTout::default(),
            "I realize it's much touted for having a brilliant API, but unfortunately that's not sufficient",
        )
    }

    #[test]
    fn fix_often_toted() {
        assert_suggestion_result(
            "The NN being often toted as a black box (even some say model free) model or function of its input vector.",
            ToteTout::default(),
            "The NN being often touted as a black box (even some say model free) model or function of its input vector.",
        )
    }

    #[test]
    fn fix_much_toted_hyphen() {
        assert_suggestion_result(
            "The much-toted \"93% hire rate\" is certainly not true for my class.",
            ToteTout::default(),
            "The much-touted \"93% hire rate\" is certainly not true for my class.",
        )
    }

    #[test]
    fn dont_flag_ad_toting() {
        assert_no_lints(
            "Uber's Ad-Toting Drones Are Heckling Drivers Stuck in Traffic.",
            ToteTout::default(),
        )
    }

    #[test]
    fn dont_flag_plenty_of_touts_around() {
        assert_no_lints(
            "there are plenty of pathetic touts around the hotel",
            ToteTout::default(),
        )
    }

    #[test]
    fn dont_flag_touting_box() {
        assert_no_lints(
            "distracting from the dysfunction by touting box office revenue as the solution for paltry royalty earnings",
            ToteTout::default(),
        )
    }

    #[test]
    fn fix_widely_toted() {
        assert_suggestion_result(
            "It is widely toted as the smart, capable and economically insightful newspaper for the white, liberal elite.",
            ToteTout::default(),
            "It is widely touted as the smart, capable and economically insightful newspaper for the white, liberal elite.",
        )
    }
}
