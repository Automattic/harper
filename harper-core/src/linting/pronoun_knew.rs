use crate::{
    Token,
    linting::{Lint, LintKind, PatternLinter, Suggestion},
    patterns::{EitherPattern, SequencePattern, WordSet},
};

pub struct PronounKnew {
    pattern: Box<dyn crate::patterns::Pattern>,
}

impl Default for PronounKnew {
    fn default() -> Self {
        let pronoun_then_new = SequencePattern::default()
            .then_pronoun()
            .then_whitespace()
            .then_any_capitalization_of("new");

        let pronoun_adverb_then_new = SequencePattern::default()
            .then_pronoun()
            .then_whitespace()
            .then(WordSet::new(&["always", "never", "also", "often"]))
            .then_whitespace()
            .then_any_capitalization_of("new");

        let combined_pattern = EitherPattern::new(vec![
            Box::new(pronoun_then_new),
            Box::new(pronoun_adverb_then_new),
        ]);

        Self {
            pattern: Box::new(combined_pattern),
        }
    }
}

impl PatternLinter for PronounKnew {
    fn pattern(&self) -> &dyn crate::patterns::Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        let typo_token = tokens.last()?;
        let typo_span = typo_token.span;
        let typo_text = typo_span.get_content(source);

        Some(Lint {
            span: typo_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "knew".chars().collect(),
                typo_text,
            )],
            message: "Did you mean “knew” (the past tense of “know”)?".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Detects when “new” following a pronoun (optionally with an adverb) is a typo for the past tense “knew.”"
    }
}

#[cfg(test)]
mod tests {
    use super::PronounKnew;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn simple_pronoun_new() {
        assert_suggestion_result(
            "I new you would say that.",
            PronounKnew::default(),
            "I knew you would say that.",
        );
    }

    #[test]
    fn with_adverb() {
        assert_suggestion_result(
            "She often new the answer.",
            PronounKnew::default(),
            "She often knew the answer.",
        );
    }

    #[test]
    fn does_not_flag_without_pronoun() {
        assert_lint_count("The software is new.", PronounKnew::default(), 0);
    }

    #[test]
    fn does_not_flag_other_context() {
        assert_lint_count("They called it \"new\".", PronounKnew::default(), 0);
    }
}
