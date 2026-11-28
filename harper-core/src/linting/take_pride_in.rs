use crate::{
    Lint, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct TakePrideIn {
    expr: All,
}

impl Default for TakePrideIn {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["take", "taken", "takes", "taking", "took"])
                .t_ws()
                .then_word_seq(&["pride", "of"])
                .but_not(
                    SequenceExpr::anything()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_ws()
                        .t_aco("place"),
                ),
        }
    }
}

impl ExprLinter for TakePrideIn {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.last()?.span;
        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "in",
                span.get_content(src),
            )],
            message: "This idiom uses the preposition `in` rather than `of`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `take pride of` to `take pride in`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::TakePrideIn;

    #[test]
    fn take() {
        assert_suggestion_result(
            "I take pride of my work and go out of my way to support people who honored me with building their product using it.",
            TakePrideIn::default(),
            "I take pride in my work and go out of my way to support people who honored me with building their product using it.",
        );
    }

    #[test]
    fn taken() {
        assert_suggestion_result(
            "... the variety and multifaceted society that we Malaysians have always taken pride of?",
            TakePrideIn::default(),
            "... the variety and multifaceted society that we Malaysians have always taken pride in?",
        );
    }

    #[test]
    fn takes() {
        assert_suggestion_result(
            "Being a part of the team that takes pride of their work.",
            TakePrideIn::default(),
            "Being a part of the team that takes pride in their work.",
        )
    }

    #[test]
    fn took() {
        assert_suggestion_result(
            "and somehow we always took pride of well written and maintainble code",
            TakePrideIn::default(),
            "and somehow we always took pride in well written and maintainble code",
        );
    }

    #[test]
    fn dont_flag_take_pride_of_place() {
        assert_no_lints(
            "Any better alternatives will take pride of place, but for now, this is fine.",
            TakePrideIn::default(),
        )
    }

    #[test]
    fn dont_flag_taken_pride_of_place() {
        assert_no_lints(
            "The venture-backed model has taken pride of place, but we should celebrate forks",
            TakePrideIn::default(),
        )
    }

    #[test]
    fn dont_flag_taken_pride_of_place_hyphenated() {
        assert_no_lints(
            "\"sometimes some strange errors might occur\" has now taken pride-of-place as my favorite bug report of all time",
            TakePrideIn::default(),
        )
    }

    #[test]
    fn dont_flag_takes_pride_of_place() {
        assert_no_lints(
            "this magnificent family residence takes pride of place just footsteps from the tranquil shores of Vaucluse Bay",
            TakePrideIn::default(),
        )
    }

    #[test]
    fn dont_flag_took_pride_of_place() {
        assert_no_lints(
            "They were built to be exceptionally robust, and that took pride of place (for a while) over technical and luxury features.",
            TakePrideIn::default(),
        )
    }

    // Known edge cases

    #[test]
    #[ignore = "Surely too rare to address"]
    fn dont_flag_edge_case() {
        assert_no_lints(
            "what is known as “dune bashing” has taken pride of honour on my list of “don’t ever do such a stupid thing again.”",
            TakePrideIn::default(),
        )
    }
}
