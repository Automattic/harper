use crate::{
    CharStringExt, TokenStringExt,
    linting::PatternLinter,
    patterns::{All, SplitCompoundWord},
};

use super::{Lint, LintKind, Suggestion};

use crate::{
    Lrc, Token,
    patterns::{Pattern, SequencePattern},
};

/// Covers the general cases of accidentally split compound nouns.
pub struct GeneralCompoundNouns {
    pattern: Box<dyn Pattern>,
    split_pattern: Lrc<SplitCompoundWord>,
}

impl Default for GeneralCompoundNouns {
    fn default() -> Self {
        let exceptions_pattern = SequencePattern::default()
            .then(|tok: &Token, _: &[char]| {
                let Some(Some(meta)) = tok.kind.as_word() else {
                    return false;
                };

                meta.article || meta.is_adjective()
            })
            .then_whitespace()
            .then(|tok: &Token, _: &[char]| {
                let Some(Some(meta)) = tok.kind.as_word() else {
                    return false;
                };

                tok.span.len() > 1 && !meta.article && !meta.preposition && !meta.is_adverb()
            })
            .then_whitespace()
            .then(|tok: &Token, _: &[char]| {
                let Some(Some(meta)) = tok.kind.as_word() else {
                    return false;
                };

                tok.span.len() > 1 && !meta.article && !meta.is_adverb() && !meta.preposition
            });

        let split_pattern = Lrc::new(SplitCompoundWord::new(|meta| {
            meta.is_noun() && !meta.is_adjective()
        }));

        let mut pattern = All::default();
        pattern.add(Box::new(exceptions_pattern));
        pattern.add(Box::new(
            SequencePattern::default()
                .then_anything()
                .then_anything()
                .then(split_pattern.clone()),
        ));

        Self {
            pattern: Box::new(pattern),
            split_pattern,
        }
    }
}

impl PatternLinter for GeneralCompoundNouns {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens[2..].span()?;
        let orig = span.get_content(source);
        // If the pattern matched, this will not return `None`.
        let word =
            self.split_pattern
                .get_merged_word(matched_tokens[2], matched_tokens[4], source)?;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(word.to_vec(), orig)],
            message: format!(
                "Did you mean the closed compound noun “{}”?",
                word.to_string()
            ),
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Detects compound nouns split by a space and suggests merging them when both parts form a valid noun. Has checks to avoid erroneous cases."
    }
}
