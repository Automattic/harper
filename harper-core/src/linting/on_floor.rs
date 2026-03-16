use crate::CharStringExt;
use crate::expr::Expr;
use crate::expr::LongestMatchOf;
use crate::expr::SequenceExpr;
use crate::linting::expr_linter::Chunk;
use crate::patterns::WordSet;
use crate::{
    Lrc, Token, TokenStringExt,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct OnFloor {
    expr: Box<dyn Expr>,
}

impl Default for OnFloor {
    fn default() -> Self {
        let preposition = WordSet::new(&["in", "at"]);

        let on_the_floor = Lrc::new(
            SequenceExpr::with(preposition)
                .t_ws()
                .t_aco("the")
                .t_ws()
                .t_any()
                .t_ws()
                .t_aco("floor"),
        );

        let look_up_phrase = Lrc::new(
            SequenceExpr::word_set(&["look", "looking", "looks", "looked"])
                .t_ws()
                .t_aco("up"),
        );

        let stop = Lrc::new(WordSet::new(&["stop", "stopping", "stops", "stopped"]));
        let exceptions = Lrc::new(LongestMatchOf::new(vec![
            Box::new(SequenceExpr::with(look_up_phrase.clone())),
            Box::new(SequenceExpr::with(stop.clone())),
        ]));

        let pattern = LongestMatchOf::new(vec![
            Box::new(on_the_floor.clone()),
            Box::new(
                SequenceExpr::with(exceptions.clone())
                    .t_ws()
                    .then(on_the_floor.clone()),
            ),
        ]);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for OnFloor {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let incorrect_preposition = matched_tokens[0..1].span()?.get_content(source).to_string();
        // if the first token is not "in" or "at", means that the match is belong to the exceptions
        // so we don't need to lint it
        if !["in", "at"].contains(&incorrect_preposition.to_lowercase().as_str()) {
            return None;
        }
        let span = matched_tokens[0..1].span()?;

        Some(Lint {
            lint_kind: LintKind::WordChoice,
            span,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "on",
                span.get_content(source),
            )],
            message: format!(
                "Corrects `{incorrect_preposition}` to `on` when talking about position inside a building",
            )
            .to_string(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "This rule identifies incorrect uses of the prepositions `in` or `at` when referring to locations inside a building and recommends using `on the floor` instead."
    }
}

#[cfg(test)]
mod tests {
    use super::OnFloor;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn not_lint_with_correct_phrase() {
        assert_lint_count(
            "I'm living on the 3rd floor of a building.",
            OnFloor::default(),
            0,
        );
    }

    #[test]
    fn lint_with_in() {
        assert_suggestion_result(
            "I'm living in the 3rd floor of a building.",
            OnFloor::default(),
            "I'm living on the 3rd floor of a building.",
        );
    }

    #[test]
    fn lint_with_at() {
        assert_suggestion_result(
            "I'm living at the second floor of a building.",
            OnFloor::default(),
            "I'm living on the second floor of a building.",
        );
    }

    #[test]
    fn in_the_start_of_sentence() {
        assert_suggestion_result(
            "In the 3rd floor of a building.",
            OnFloor::default(),
            "On the 3rd floor of a building.",
        );
    }

    #[test]
    fn at_the_start_of_sentence() {
        assert_suggestion_result(
            "At the second floor of a building.",
            OnFloor::default(),
            "On the second floor of a building.",
        );
    }

    #[test]
    fn no_lint_with_look_up_at() {
        assert_lint_count("She looked up at the third floor.", OnFloor::default(), 0);
    }

    #[test]
    fn no_lint_with_stop_at() {
        assert_lint_count(
            "The elevator stops at the 3rd floor of a building.",
            OnFloor::default(),
            0,
        );
    }

    #[test]
    fn no_lint_with_looking_up_at() {
        assert_lint_count(
            "The workers are looking up at the 3rd floor of a building.",
            OnFloor::default(),
            0,
        );
    }
}
