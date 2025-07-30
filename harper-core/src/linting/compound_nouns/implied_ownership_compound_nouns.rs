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
/// See also:
/// harper-core/src/linting/lets_confusion/mod.rs
/// harper-core/src/linting/lets_confusion/let_us_redundancy.rs
/// harper-core/src/linting/lets_confusion/no_contraction_with_verb.rs
/// harper-core/src/linting/pronoun_contraction/should_contract.rs
pub struct ImpliedOwnershipCompoundNouns {
    pattern: Box<dyn Pattern>,
    split_pattern: Lrc<SplitCompoundWord>,
}

impl Default for ImpliedOwnershipCompoundNouns {
    fn default() -> Self {
        let split_pattern = Lrc::new(SplitCompoundWord::new(|meta| meta.is_noun()));
        let pattern = SequencePattern::default()
            .then_possessive_nominal()
            .then_whitespace()
            .then(split_pattern.clone());

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

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        // "Let's" can technically be a possessive noun (of a lease, or a let in tennis, etc.)
        // but in practice it's almost always a contraction of "let us" before a verb
        // or a mistake for "lets", the 3rd person singular present form of "to let".
        let word_apostrophe_s = matched_tokens[0].span.get_content(source);
        if word_apostrophe_s
            .iter()
            .map(|&c| c.to_ascii_lowercase())
            .eq(['l', 'e', 't', '\'', 's'].iter().copied())
        {
            return None;
        }
        let span = matched_tokens[2..].span()?;
        // If the pattern matched, this will not return `None`.
        let word =
            self.split_pattern
                .get_merged_word(&matched_tokens[2], &matched_tokens[4], source)?;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith(word.to_vec())],
            message: format!(
                "The possessive noun implies ownership of the closed compound noun “{}”.",
                word.to_string()
            ),
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Detects split compound nouns following a possessive noun and suggests merging them."
    }
}

#[cfg(test)]
mod tests {
    use super::ImpliedOwnershipCompoundNouns;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn does_not_flag_lets() {
        assert_lint_count(
            "Let's check out this article.",
            ImpliedOwnershipCompoundNouns::default(),
            0,
        );
    }
}
