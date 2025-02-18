use super::{Lint, LintKind, PatternLinter};
use crate::linting::Suggestion;
use crate::patterns::{Pattern, SequencePattern, WordSet};
use crate::Token;
use crate::TokenStringExt;

pub struct WasAloud {
    pattern: Box<dyn Pattern>,
}

impl Default for WasAloud {
    fn default() -> Self {
        let pattern = SequencePattern::default()
            .then_word_set(WordSet::all(&["was", "were", "be", "been"]))
            .then_whitespace()
            .then_exact_word("aloud");

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for WasAloud {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let verb = matched_tokens[0].span.get_content_string(source);

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                format!("{} allowed", verb).chars().collect(),
                matched_tokens[0].span.get_content(source),
            )],
            message: format!("Did you mean `{verb} allowed`?"),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Ensures `was aloud` and `were aloud` are corrected to `was allowed` or `were allowed` when referring to permission."
    }
}

#[cfg(test)]
mod tests {
    use super::WasAloud;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn corrects_was_aloud() {
        assert_suggestion_result(
            "He was aloud to enter the room.",
            WasAloud::default(),
            "He was allowed to enter the room.",
        );
    }

    #[test]
    fn corrects_were_aloud() {
        assert_suggestion_result(
            "They were aloud to participate.",
            WasAloud::default(),
            "They were allowed to participate.",
        );
    }

    #[test]
    fn does_not_correct_proper_use_of_aloud() {
        assert_suggestion_result(
            "She read the passage aloud to the class.",
            WasAloud::default(),
            "She read the passage aloud to the class.",
        );
    }

    #[test]
    fn does_not_flag_unrelated_text() {
        assert_suggestion_result(
            "The concert was loud and exciting.",
            WasAloud::default(),
            "The concert was loud and exciting.",
        );
    }

    #[test]
    fn be_aloud() {
        assert_suggestion_result(
            "You may be aloud to enter the room.",
            WasAloud::default(),
            "You may be allowed to enter the room.",
        );
    }

    #[test]
    fn been_aloud() {
        assert_suggestion_result(
            "If I had been aloud to enter I would've jumped at the chance.",
            WasAloud::default(),
            "If I had been allowed to enter I would've jumped at the chance.",
        );
    }
}
