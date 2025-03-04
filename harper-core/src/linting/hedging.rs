use crate::linting::{Lint, LintKind, PatternLinter};
use crate::patterns::{EitherPattern, ExactPhrase, Pattern};
use crate::{Token, TokenStringExt};

/// A linter that detects hedging language.
pub struct Hedging {
    pattern: Box<dyn Pattern>,
}

impl Default for Hedging {
    fn default() -> Self {
        let phrases = vec!["I would argue that", ", so to speak", "to a certain degree"];

        let patterns: Vec<Box<dyn Pattern>> = phrases
            .into_iter()
            .map(|s| Box::new(ExactPhrase::from_phrase(s)) as Box<dyn Pattern>)
            .collect();

        let pattern = Box::new(EitherPattern::new(patterns));
        Self { pattern }
    }
}

impl PatternLinter for Hedging {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: Vec::new(),
            message: "You're hedging.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Flags hedging language (e.g. `I would argue that`, `..., so to speak`, `to a certain degree`)."
    }
}

#[cfg(test)]
mod tests {
    use super::Hedging;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn detects_hedging_phrase() {
        assert_lint_count("I would argue that this is correct.", Hedging::default(), 1);
    }

    #[test]
    fn does_not_flag_clean_text() {
        assert_lint_count("This is clear and direct.", Hedging::default(), 0);
    }

    #[test]
    fn lowercase_hedging() {
        assert_lint_count(
            "i would argue that the outcome is uncertain.",
            Hedging::default(),
            1,
        );
    }

    #[test]
    fn incomplete_phrase_not_flagged() {
        assert_lint_count("I would argue the data is clear.", Hedging::default(), 0);
    }

    #[test]
    fn phrase_with_trailing_comma() {
        let text = "I would argue that, this method works.";
        assert_lint_count(text, Hedging::default(), 1);
    }

    #[test]
    fn phrase_with_extra_whitespace() {
        assert_lint_count(
            "to   a   certain   degree the results are ambiguous.",
            Hedging::default(),
            1,
        );
    }

    #[test]
    fn does_not_flag_similar_but_incorrect_phrase() {
        assert_lint_count(
            "He spoke so to speakingly about the event.",
            Hedging::default(),
            0,
        );
    }

    #[test]
    fn phrase_split_by_line_break() {
        assert_lint_count(
            "I would argue\nthat this approach fails.",
            Hedging::default(),
            1,
        );
    }
}
