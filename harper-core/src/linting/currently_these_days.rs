use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct CurrentlyTheseDays {
    expr: FirstMatchOf,
}

impl Default for CurrentlyTheseDays {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::aco("currently")
                        .t_ws()
                        .then_zero_or_more(
                            SequenceExpr::default().then_word_except(&["these"]).t_ws(),
                        )
                        .then_word_seq(&["these", "days"]),
                ),
                Box::new(
                    SequenceExpr::word_seq(&["these", "days"])
                        .t_ws()
                        .then_zero_or_more(
                            SequenceExpr::default()
                                .then_word_except(&["currently"])
                                .t_ws(),
                        )
                        .t_aco("currently"),
                ),
            ]),
        }
    }
}

enum Order {
    CurrentlyTheseDays,
    TheseDaysCurrently,
}
use Order::*;

impl ExprLinter for CurrentlyTheseDays {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let order = if toks[0].get_ch(src).eq_str("currently") {
            CurrentlyTheseDays
        } else {
            TheseDaysCurrently
        };

        let (from_start, from_end) = match order {
            CurrentlyTheseDays => (2, 4),
            TheseDaysCurrently => (4, 2),
        };

        let (sug1, sug2) = (
            toks.get_rel_slice(from_start, -1)?,
            toks.get_rel_slice(0, -from_end - 1)?,
        );

        let span = toks.span()?;

        let suggestions = vec![
            Suggestion::replace_with_match_case(sug1.get_ch(src)?.to_vec(), span.get_content(src)),
            Suggestion::replace_with_match_case(sug2.get_ch(src)?.to_vec(), span.get_content(src)),
        ];

        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions,
            message: "Use either `currently` or `these days` but not both.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects redundant use of `currently` and `these days` together."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_good_and_bad_suggestions;

    use super::CurrentlyTheseDays;

    #[test]
    fn holotype() {
        assert_good_and_bad_suggestions(
            "Now this app is called Floorplan, and it's an open source project and is currently getting a lot of traction these days.",
            CurrentlyTheseDays::default(),
            &[
                "Now this app is called Floorplan, and it's an open source project and is currently getting a lot of traction.",
                "Now this app is called Floorplan, and it's an open source project and is getting a lot of traction these days.",
            ],
            &[],
        );
    }
}
