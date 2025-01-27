use crate::linting::{LintKind, PatternLinter, Suggestion};
use crate::patterns::{EitherPattern, Invert, Pattern, SequencePattern, WordPatternGroup};
use crate::{Lint, Lrc, Token};

// Looks for places where the genitive case _isn't_ being used, and should be.
pub struct UseGenitive {
    pattern: Box<dyn Pattern>,
}

impl UseGenitive {
    fn new() -> Self {
        // Define the environment in which the genitive case __should__ be used.
        let environment = Lrc::new(SequencePattern::default().then_whitespace().then(Box::new(
            EitherPattern::new(vec![
                    Box::new(
                        SequencePattern::default()
                            .then_one_or_more_adjectives()
                            .then_whitespace()
                            .then_noun(),
                    ),
                    Box::new(SequencePattern::default().then_noun()),
                ]),
        )));

        let trigger_words = ["there", "they're"];

        let mut primary_pattern = WordPatternGroup::default();

        for word in trigger_words {
            primary_pattern.add(
                word,
                Box::new(
                    SequencePattern::default()
                        .then_exact_word(word)
                        .then(Box::new(environment.clone())),
                ),
            )
        }

        // Add a prelude to remove false-positives.
        let full_pattern = SequencePattern::default()
            .then(Box::new(Invert::new(Box::new(EitherPattern::new(vec![
                Box::new(SequencePattern::default().then_exact_word_or_lowercase("Is")),
                Box::new(SequencePattern::default().then_exact_word_or_lowercase("Were")),
                Box::new(SequencePattern::default().then_adjective()),
            ])))))
            .then_whitespace()
            .then(Box::new(primary_pattern));

        Self {
            pattern: Box::new(full_pattern),
        }
    }
}

impl PatternLinter for UseGenitive {
    fn pattern(&self) -> &dyn crate::patterns::Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Lint {
        Lint {
            span: matched_tokens[2].span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith(vec!['t', 'h', 'e', 'i', 'r'])],
            message: "Use the genitive case.".to_string(),
            priority: 31,
        }
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
