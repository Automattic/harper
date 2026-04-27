use crate::{
    Lint, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct ArriveTo {
    expr: All,
}

impl Default for ArriveTo {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["arrive", "arrived", "arrives", "arriving"])
                .t_ws()
                .t_aco("to")
                .and_not(
                    SequenceExpr::anything()
                        .t_any()
                        .t_any()
                        .t_any()
                        .then_verb_lemma(),
                ),
        }
    }
}

impl ExprLinter for ArriveTo {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let prep_idx = 2;
        let span = toks[prep_idx].span;

        let suggestions = ["at", "in"]
            .iter()
            .map(|&s| Suggestion::replace_with_match_case_str(s, span.get_content(src)))
            .collect();

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "If the noun is a destination, use 'at' or 'in' instead of 'to'.".to_string(),
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
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ArriveTo;

    // Basic functionality tests

    #[test]
    fn fix_with_destination() {
        assert_suggestion_result(
            "arrive to the destination",
            ArriveTo::default(),
            "arrive at the destination",
        );
    }

    #[test]
    fn fix_without_destination() {
        assert_suggestion_result("arrive to", ArriveTo::default(), "arrive at");
    }

    #[test]
    fn dont_flag_with_verb_lemma() {
        assert_no_lints(
            "I arrived to find something was wrong.",
            ArriveTo::default(),
        );
    }
    // Tests using real-world sentences from the Internt.

    #[test]
    fn fix_flights_example() {
        assert_suggestion_result(
            "Number of flights departing from and arriving to every airport.",
            ArriveTo::default(),
            "Number of flights departing from and arriving at every airport.",
        );
    }

    #[test]
    fn fix_vancouver_example() {
        assert_suggestion_result(
            "We had several days vacation in Vancouver prior to arriving to Seattle",
            ArriveTo::default(),
            "We had several days vacation in Vancouver prior to arriving in Seattle",
        );
    }

    #[test]
    fn fix_packet_example() {
        assert_suggestion_result(
            "The packet arrives to the client socket correctly as well, according to Wireshark.",
            ArriveTo::default(),
            "The packet arrives at the client socket correctly as well, according to Wireshark.",
        );
    }

    #[test]
    fn fix_email_example() {
        assert_suggestion_result(
            "Okay, so, add whatever email address the emails arrive to originally as a System Email under Admin Panel",
            ArriveTo::default(),
            "Okay, so, add whatever email address the emails arrive at originally as a System Email under Admin Panel",
        );
    }

    #[test]
    fn fix_game_menu_example() {
        assert_suggestion_result(
            "God Of War Collection: White screen before arrive to the game menu",
            ArriveTo::default(),
            "God Of War Collection: White screen before arrive at the game menu",
        );
    }

    #[test]
    fn dont_flag_customer_arrives() {
        assert_no_lints(
            "whenever a new customer arrives to determine which set they will belong to",
            ArriveTo::default(),
        );
    }

    #[test]
    fn dont_flag_luxury_bus_arrived() {
        assert_no_lints(
            "Another luxury bus has arrived to collect arrested protesters chilling on the guardrail.",
            ArriveTo::default(),
        );
    }

    #[test]
    fn dont_flag_soviet_paratroopers_arrived() {
        assert_no_lints(
            "On the morning of 28 December Soviet paratroopers arrived to protect the hotel where he was staying.",
            ArriveTo::default(),
        );
    }

    #[test]
    fn dont_flag_negotiator_arrives() {
        assert_no_lints(
            "US negotiator with North Korea arrives to begin talks.",
            ArriveTo::default(),
        );
    }
}
