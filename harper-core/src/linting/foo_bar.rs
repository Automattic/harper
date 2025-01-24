use itertools::Itertools;

use crate::{
    patterns::{Pattern, SequencePattern, WordPatternGroup},
    Lrc, Token, TokenStringExt,
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct FooBar {
    pattern: Box<dyn Pattern>,
}

impl Default for FooBar {
    fn default() -> Self {
        let mut pattern = WordPatternGroup::default();

        let matching_pattern = Lrc::new(
            SequencePattern::default()
                .then_exact_word_or_lowercase("Foo")
                .then_whitespace()
                .then_exact_word("foo"),
        );

        pattern.add("foo", Box::new(matching_pattern.clone()));
        pattern.add("Foo", Box::new(matching_pattern));

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for FooBar {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
        let suggestion = format!(
            "{} bar",
            matched_tokens[0]
                .span
                .get_content(source)
                .iter()
                .collect::<String>()
        )
        .chars()
        .collect_vec();

        Lint {
            span: matched_tokens.span().unwrap(),
            lint_kind: LintKind::Repetition,
            suggestions: vec![Suggestion::ReplaceWith(suggestion)],
            message: "“foo foo” sometimes means “foo bar”, bar is clearer.".to_string(),
            priority: 126,
        }
    }

    fn description(&self) -> &'static str {
        "Repeating the word \"foo\" twice is often redundant. `Foo bar` is easier to read."
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::assert_lint_count;
    use super::FooBar;

    #[test]
    fn catches_lowercase() {
        assert_lint_count(
            "To reiterate, foo foo is cool is not uncool.",
            FooBar::default(),
            1,
        );
    }

    #[test]
    fn catches_different_cases() {
        assert_lint_count("Foo foo is cool is not uncool.", FooBar::default(), 1);
    }

    #[test]
    fn likes_correction() {
        assert_lint_count(
            "To reiterate, foo bar is cool is not uncool.",
            FooBar::default(),
            0,
        );
    }
}
