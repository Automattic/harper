use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

pub struct MeansALotTo {
    expr: SequenceExpr,
}

impl Default for MeansALotTo {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                // Note that "meaning a lot to" is not used.
                .t_set(&["mean", "means", "meant"])
                .t_ws()
                .then_any_of(vec![
                    Box::new(FixedPhrase::from_phrase("a lot")),
                    Box::new(WordSet::new(&["alot", "lot"])),
                ])
                .t_ws()
                .t_aco("for"),
        }
    }
}

impl ExprLinter for MeansALotTo {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if ![5, 7].contains(&toks.len()) {
            return None;
        }

        let (span, sug, msg) = match toks.len() {
            5 => (
                toks[2..5].span()?,
                "a lot to",
                "In this phrase the correct spelling is `a lot` and the correct preposition is `to`.",
            ),
            7 => (
                toks.last()?.span,
                "to",
                "The correct preposition in this phrase is `to`.",
            ),
            _ => return None,
        };
        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                sug.chars().collect(),
                span.get_content(src),
            )],
            message: msg.to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects wrong variants of `means a lot for [someone]` to `means a lot to [someone]`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::MeansALotTo;

    #[test]
    fn fix_mean_a_lot_for() {
        assert_suggestion_result(
            "It would mean a lot for me and save me a lot of time and effort",
            MeansALotTo::default(),
            "It would mean a lot to me and save me a lot of time and effort",
        );
    }

    #[test]
    fn fix_mean_alot_for() {
        assert_suggestion_result(
            "Appreciate it! Would mean alot for me!",
            MeansALotTo::default(),
            "Appreciate it! Would mean a lot to me!",
        );
    }

    #[test]
    fn fix_mean_lot_for() {
        assert_suggestion_result(
            "It would be very grateful to achieve number of sponsors for me and it mean lot for me.",
            MeansALotTo::default(),
            "It would be very grateful to achieve number of sponsors for me and it mean a lot to me.",
        );
    }

    #[test]
    fn fix_means_a_lot_for() {
        assert_suggestion_result(
            "Your star means a lot for us to develop this project!",
            MeansALotTo::default(),
            "Your star means a lot to us to develop this project!",
        );
    }

    #[test]
    fn fix_means_alot_for() {
        assert_suggestion_result(
            "Even a single sponsor means alot for me.",
            MeansALotTo::default(),
            "Even a single sponsor means a lot to me.",
        );
    }

    #[test]
    fn fix_means_lot_for() {
        assert_suggestion_result(
            "Any help means lot for me.",
            MeansALotTo::default(),
            "Any help means a lot to me.",
        );
    }

    #[test]
    fn fix_meant_a_lot_for() {
        assert_suggestion_result(
            "It meant a lot for me to improve this library further.",
            MeansALotTo::default(),
            "It meant a lot to me to improve this library further.",
        );
    }

    #[test]
    fn fix_meant_alot_for() {
        assert_suggestion_result(
            "Thanks a lot by the way. this meant alot for me.",
            MeansALotTo::default(),
            "Thanks a lot by the way. this meant a lot to me.",
        );
    }
}
