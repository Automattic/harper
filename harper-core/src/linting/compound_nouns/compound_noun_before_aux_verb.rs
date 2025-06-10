use crate::{
    CharStringExt, Lrc, TokenStringExt,
    linting::PatternLinter,
    patterns::{All, MergeableWords},
};

use super::{Lint, LintKind, Suggestion, is_content_word, predicate};

use crate::{
    Token,
    patterns::{Pattern, SequencePattern},
};

/// Two adjacent words separated by whitespace that if joined would be a valid noun.
pub struct CompoundNounBeforeAuxVerb {
    pattern: Box<dyn Pattern>,
    split_pattern: Lrc<MergeableWords>,
}

impl Default for CompoundNounBeforeAuxVerb {
    fn default() -> Self {
        let context_pattern = SequencePattern::default()
            .then(is_content_word)
            .t_ws()
            .then(is_content_word)
            .then_auxiliary_verb();

        let split_pattern = Lrc::new(MergeableWords::new(|meta_closed, meta_open| {
            predicate(meta_closed, meta_open)
        }));

        let mut pattern = All::default();
        pattern.add(Box::new(context_pattern));
        pattern.add(Box::new(
            SequencePattern::default()
                .then(split_pattern.clone())
                .then_anything()
                .then_anything(),
        ));

        Self {
            pattern: Box::new(pattern),
            split_pattern,
        }
    }
}

impl PatternLinter for CompoundNounBeforeAuxVerb {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens[0..3].span()?;
        let orig = span.get_content(source);
        // If the pattern matched, this will not return `None`.
        let word =
            self.split_pattern
                .get_merged_word(&matched_tokens[0], &matched_tokens[2], source)?;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(word.to_vec(), orig)],
            message: format!(
                "The auxiliary verb “{}” implies the existence of the closed compound noun “{}”.",
                matched_tokens[4].span.get_content(source).to_string(),
                word.to_string()
            ),
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Detects split compound nouns preceding an action and suggests merging them."
    }
}
