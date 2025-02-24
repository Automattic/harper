use crate::{
    CharStringExt, Lrc, TokenStringExt, linting::PatternLinter, patterns::SplitCompoundWord,
};

use super::{Lint, LintKind, Suggestion};

use crate::{
    Token,
    patterns::{Pattern, SequencePattern},
};

/// Looks for closed compound nouns which can be condensed due to their position after a
/// possessive noun (which implies ownership).
pub struct ImpliedActionCompoundNouns {
    pattern: Box<dyn Pattern>,
    split_pattern: Lrc<SplitCompoundWord>,
}

impl Default for ImpliedActionCompoundNouns {
    fn default() -> Self {
        let split_pattern = Lrc::new(SplitCompoundWord::new(|meta| meta.is_noun()));
        let pattern = SequencePattern::default()
            .then(split_pattern.clone())
            .then_whitespace()
            .then(|tok: &Token, _source: &[char]| {
                tok.kind.is_verb() && !tok.kind.is_likely_homograph()
            });

        Self {
            pattern: Box::new(pattern),
            split_pattern,
        }
    }
}

impl PatternLinter for ImpliedActionCompoundNouns {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens[0..3].span()?;
        let orig = span.get_content(source);
        // If the pattern matched, this will not return `None`.
        let word =
            self.split_pattern
                .get_merged_word(matched_tokens[0], matched_tokens[2], source)?;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(word.to_vec(), orig)],
            message: format!(
                "The verb here implies the existence of the closed compound noun “{}”.",
                word.to_string()
            ),
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Detects split compound nouns preceding an action and suggests merging them."
    }
}
