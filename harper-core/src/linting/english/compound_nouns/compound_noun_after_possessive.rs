use crate::expr::All;
use crate::expr::Expr;
use crate::expr::MergeableWords;
use crate::expr::SequenceExpr;
use crate::{CharStringExt, Lrc, TokenStringExt, linting::ExprLinter};

use super::{Lint, LintKind, Suggestion, is_content_word, predicate};
use crate::linting::expr_linter::Chunk;

use crate::Token;

/// Looks for closed compound nouns which can be condensed due to their position after a
/// possessive noun (which implies ownership).
/// See also:
/// harper-core/src/linting/lets_confusion/mod.rs
/// harper-core/src/linting/lets_confusion/let_us_redundancy.rs
/// harper-core/src/linting/lets_confusion/no_contraction_with_verb.rs
/// harper-core/src/linting/pronoun_contraction/should_contract.rs
pub struct CompoundNounAfterPossessive {
    expr: Box<dyn Expr>,
    split_pattern: Lrc<MergeableWords>,
}

impl Default for CompoundNounAfterPossessive {
    fn default() -> Self {
        let context_pattern = SequenceExpr::default()
            .then_possessive_nominal()
            .t_ws()
            .then(is_content_word)
            .t_ws()
            .then(is_content_word);

        let split_pattern = Lrc::new(MergeableWords::new(|meta_closed, meta_open| {
            predicate(meta_closed, meta_open)
        }));

        let mut pattern = All::default();

        pattern.add(context_pattern);
        pattern.add(SequenceExpr::anything().t_any().then(split_pattern.clone()));

        Self {
            expr: Box::new(pattern),
            split_pattern,
        }
    }
}

impl ExprLinter for CompoundNounAfterPossessive {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        // "Let's" can technically be a possessive noun (of a lease, or a let in tennis, etc.)
        // but in practice it's almost always a contraction of "let us" before a verb
        // or a mistake for "lets", the 3rd person singular present form of "to let".
        let word_apostrophe_s = matched_tokens[0]
            .span
            .get_content_string(source)
            .to_lowercase()
            .replace('’', "'");
        if word_apostrophe_s == "let's" || word_apostrophe_s == "that's" {
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
    use super::CompoundNounAfterPossessive;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn lets_is_not_possessive() {
        assert_lint_count(
            "Let's check out this article.",
            CompoundNounAfterPossessive::default(),
            0,
        );
    }

    #[test]
    fn lets_is_not_possessive_typographic_apostrophe() {
        assert_lint_count(
            "“Let’s go on with the game,” the Queen said to Alice;",
            CompoundNounAfterPossessive::default(),
            0,
        )
    }

    #[test]
    fn thats_is_not_possessive() {
        assert_lint_count(
            "And you might not be thinking that that's a very big issue, but ...",
            CompoundNounAfterPossessive::default(),
            0,
        );
    }
}
