use crate::{
    linting::PatternLinter, patterns::SplitCompoundWord, CharStringExt, Lrc, TokenStringExt,
};

use super::{Lint, LintKind, Suggestion};

use crate::{
    patterns::{Pattern, SequencePattern},
    Token,
};

/// Looks for closed compound nouns which can be condensed due to their position after a
/// possessive noun (which implies ownership).
pub struct ImpliedOwnershipCompoundNouns {
    pattern: Box<dyn Pattern>,
    split_pattern: Lrc<SplitCompoundWord>,
}

impl Default for ImpliedOwnershipCompoundNouns {
    fn default() -> Self {
        let split_pattern = Lrc::new(SplitCompoundWord::new(|meta| meta.is_noun()));
        let pattern = SequencePattern::default()
            .then_possessive_noun()
            .then_whitespace()
            .then(Box::new(split_pattern.clone()));

        Self {
            pattern: Box::new(pattern),
            split_pattern,
        }
    }
}

impl PatternLinter for ImpliedOwnershipCompoundNouns {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
        let span = matched_tokens[2..].span().unwrap();
        // If the pattern matched, this will not return `None`.
        let word = self
            .split_pattern
            .get_merged_word(matched_tokens[2], matched_tokens[4], source)
            .unwrap();

        Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith(word.to_vec())],
            message: format!(
                "The possessive noun implies ownership of the closed compound noun “{}”.",
                word.to_string()
            ),
            priority: 63,
        }
    }

    fn description(&self) -> &str {
        "Detects split compound nouns following a possessive noun and suggests merging them."
    }
}
