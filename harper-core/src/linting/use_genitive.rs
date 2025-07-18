use crate::expr::Expr;
use crate::expr::FirstMatchOf;
use crate::expr::SequenceExpr;
use crate::expr::WordExprGroup;
use crate::linting::{ExprLinter, LintKind, Suggestion};
use crate::patterns::Word;
use crate::{Lint, Lrc, Token};

// Looks for places where the genitive case _isn't_ being used, and should be.
pub struct UseGenitive {
    expr: Box<dyn Expr>,
}

impl UseGenitive {
    fn new() -> Self {
        // Define the environment in which the genitive case __should__ be used.
        let environment = Lrc::new(SequenceExpr::default().then_whitespace().then_longest_of(
            vec![
                    Box::new(
                        SequenceExpr::default()
                            .then_one_or_more_adjectives()
                            .then_whitespace()
                            .then_noun(),
                    ),
                    Box::new(SequenceExpr::default().then_noun()),
                ],
        ));

        let trigger_words = ["there", "they're"];

        let mut primary_pattern = WordExprGroup::default();

        for word in trigger_words {
            primary_pattern.add(
                word,
                SequenceExpr::default()
                    .then(Word::new_exact(word))
                    .then(environment.clone()),
            )
        }

        // Add a prelude to remove false-positives.
        let full_pattern = SequenceExpr::default()
            .then_unless(FirstMatchOf::new(vec![
                Box::new(SequenceExpr::default().t_aco("is")),
                Box::new(SequenceExpr::default().t_aco("were")),
                Box::new(SequenceExpr::default().then_adjective()),
            ]))
            .then_whitespace()
            .then(primary_pattern);

        Self {
            expr: Box::new(full_pattern),
        }
    }
}

impl ExprLinter for UseGenitive {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        Some(Lint {
            span: matched_tokens[2].span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith(vec!['t', 'h', 'e', 'i', 'r'])],
            message: "Use the genitive case.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Looks for situations where the genitive case of \"there\" should be used."
    }
}

impl Default for UseGenitive {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::UseGenitive;

    #[test]
    fn catches_adjective_noun() {
        assert_suggestion_result(
            "What are there big problems?",
            UseGenitive::default(),
            "What are their big problems?",
        )
    }

    #[test]
    fn catches_just_noun() {
        assert_suggestion_result(
            "What are there problems?",
            UseGenitive::default(),
            "What are their problems?",
        )
    }

    #[test]
    fn allows_clause_termination() {
        assert_lint_count("Look there!", UseGenitive::default(), 0)
    }

    #[test]
    fn allows_there_are() {
        assert_lint_count(
            "Since there are people here, we should be socially aware.",
            UseGenitive::default(),
            0,
        )
    }

    #[test]
    fn allows_there_at_beginning() {
        assert_lint_count(
            "There is a cute cat sitting on the chair at home.",
            UseGenitive::default(),
            0,
        )
    }

    #[test]
    #[should_panic] // currently fails, because "received" is ambiguous between verb and adjective
    fn catches_they_are() {
        assert_suggestion_result(
            "The students received they're test results today.",
            UseGenitive::default(),
            "The students received their test results today.",
        )
    }

    #[test]
    fn allows_grantlemons_issue_267_cat() {
        assert_lint_count("Were there cats at her house?", UseGenitive::default(), 0);
    }

    #[test]
    fn allows_grantlemons_issue_267_apple() {
        assert_lint_count(
            "Were there any apples at the store?",
            UseGenitive::default(),
            0,
        );
    }

    #[test]
    fn allows_grantlemons_issue_267_fruit() {
        assert_lint_count(
            "Were there many kinds of fruit at the store?",
            UseGenitive::default(),
            0,
        );
    }

    #[test]
    fn allows_grantlemons_issue_267_people() {
        assert_lint_count(
            "Were there more than, or less than six people at the party?",
            UseGenitive::default(),
            0,
        );
    }

    #[test]
    fn allows_faster_at_running() {
        assert_lint_count(
            "Melissa was faster at running than her friend.",
            UseGenitive::default(),
            0,
        );
    }
}
