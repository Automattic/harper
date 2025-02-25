use super::Suggestion;
use super::pattern_linter::PatternLinter;
use crate::linting::LintKind;
use crate::patterns::{Pattern, SequencePattern, WordSet};
use crate::{Lint, Lrc, Token, TokenStringExt};

/// Linter that checks if multiple pronouns are being used right after each
/// other. This is a common mistake to make during the revision process.
pub struct MultipleSequentialPronouns {
    pattern: Box<dyn Pattern>,
}

impl MultipleSequentialPronouns {
    fn new() -> Self {
        let pronouns = Lrc::new(WordSet::new(&[
            "me", "my", "I", "we", "you", "he", "him", "her", "she", "it", "they",
        ]));

        Self {
            pattern: Box::new(
                SequencePattern::default()
                    .then(pronouns.clone())
                    .then_one_or_more(
                        SequencePattern::default()
                            .then_whitespace()
                            .then(pronouns.clone()),
                    ),
            ),
        }
    }
}

impl PatternLinter for MultipleSequentialPronouns {
    fn pattern(&self) -> &dyn crate::patterns::Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let mut suggestions = Vec::new();

        if matched_tokens.len() == 3 {
            suggestions.push(Suggestion::ReplaceWith(
                matched_tokens[0].span.get_content(source).to_vec(),
            ));
            suggestions.push(Suggestion::ReplaceWith(
                matched_tokens[2].span.get_content(source).to_vec(),
            ));
        }

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Repetition,
            message: "There are too many personal pronouns in sequence here.".to_owned(),
            priority: 63,
            suggestions,
        })
    }

    fn description(&self) -> &'static str {
        "When editing work to change point of view (i.e. first-person or third-person) it is common to add pronouns while neglecting to remove old ones. This rule catches cases where you have multiple disparate pronouns in sequence."
    }
}

impl Default for MultipleSequentialPronouns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::MultipleSequentialPronouns;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn can_detect_two_pronouns() {
        assert_lint_count(
            "...little bit about my I want to do.",
            MultipleSequentialPronouns::new(),
            1,
        )
    }

    #[test]
    fn can_detect_three_pronouns() {
        assert_lint_count(
            "...little bit about my I you want to do.",
            MultipleSequentialPronouns::new(),
            1,
        )
    }

    #[test]
    fn allows_single_pronouns() {
        assert_lint_count(
            "...little bit about I want to do.",
            MultipleSequentialPronouns::new(),
            0,
        )
    }

    #[test]
    fn detects_multiple_pronouns_at_end() {
        assert_lint_count(
            "...little bit about I want to do to me you.",
            MultipleSequentialPronouns::new(),
            1,
        )
    }

    #[test]
    fn comma_separated() {
        assert_lint_count("To prove it, we...", MultipleSequentialPronouns::new(), 0)
    }
}
