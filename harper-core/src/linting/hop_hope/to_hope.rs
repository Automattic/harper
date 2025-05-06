use super::super::{Lint, LintKind, PatternLinter};
use crate::linting::Suggestion;
use crate::patterns::{Pattern, SequencePattern, WordSet};
use crate::{Token, char_string::char_string};

pub struct ToHope {
    pattern: Box<dyn Pattern>,
}

impl Default for ToHope {
    fn default() -> Self {
        let pattern = SequencePattern::default()
            .then_not_plural_nominal()
            .then_whitespace()
            .then(WordSet::new(&["hop", "hopped"]))
            .then_whitespace()
            .then_nominal();

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for ToHope {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_word = &matched_tokens[2];
        let word_chars = offending_word.span.get_content(source);

        Some(Lint {
            span: offending_word.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                char_string!("hope").to_vec(),
                word_chars,
            )],
            message: "Did you mean to use 'hope' instead of 'hop' in this context?".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects incorrect use of 'hop' when the correct verb 'hope' should be used in a sentence."
    }
}
