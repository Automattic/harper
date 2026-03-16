use std::sync::Arc;

use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenKind, TokenStringExt,
    expr::{AnchorStart, Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

/// Suggests hyphenating the past tense of `roller-skate`.
pub struct RollerSkated {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl RollerSkated {
    fn roller_pair() -> SequenceExpr {
        SequenceExpr::default()
            .t_aco("roller")
            .t_ws()
            .t_aco("skated")
    }
}

impl Default for RollerSkated {
    fn default() -> Self {
        let mut map = ExprMap::default();

        map.insert(
            SequenceExpr::default()
                .then_kind_is_but_is_not(
                    |kind| matches!(kind, TokenKind::Word(_)),
                    |kind| kind.is_determiner(),
                )
                .then_whitespace()
                .then_seq(Self::roller_pair()),
            2,
        );

        map.insert(
            SequenceExpr::default()
                .then_punctuation()
                .then_whitespace()
                .then_seq(Self::roller_pair()),
            2,
        );

        map.insert(
            SequenceExpr::default()
                .then_punctuation()
                .then_seq(Self::roller_pair()),
            1,
        );

        map.insert(
            SequenceExpr::with(AnchorStart).then_seq(Self::roller_pair()),
            0,
        );

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for RollerSkated {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let roller_idx = *self.map.lookup(0, matched_tokens, source)?;
        let skated_idx = roller_idx.checked_add(2)?;
        let window = matched_tokens.get(roller_idx..=skated_idx)?;
        let span = window.span()?;
        let original = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::Punctuation,
            suggestions: vec![Suggestion::replace_with_match_case(
                "roller-skated".chars().collect(),
                original,
            )],
            message: "Hyphenate this verb as `roller-skated`.".to_owned(),
            priority: 40,
        })
    }

    fn description(&self) -> &'static str {
        "Encourages hyphenating the past tense of `roller-skate`."
    }
}

#[cfg(test)]
mod tests {
    use super::RollerSkated;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_basic_sentence() {
        assert_suggestion_result(
            "He roller skated down the hill.",
            RollerSkated::default(),
            "He roller-skated down the hill.",
        );
    }

    #[test]
    fn corrects_with_adverb() {
        assert_suggestion_result(
            "They roller skated quickly around the rink.",
            RollerSkated::default(),
            "They roller-skated quickly around the rink.",
        );
    }

    #[test]
    fn corrects_with_auxiliary() {
        assert_suggestion_result(
            "She had roller skated there before.",
            RollerSkated::default(),
            "She had roller-skated there before.",
        );
    }

    #[test]
    fn corrects_with_contraction() {
        assert_suggestion_result(
            "They'd roller skated all night.",
            RollerSkated::default(),
            "They'd roller-skated all night.",
        );
    }

    #[test]
    fn corrects_caps() {
        assert_suggestion_result(
            "They ROLLER SKATED yesterday.",
            RollerSkated::default(),
            "They ROLLER-SKATED yesterday.",
        );
    }

    #[test]
    fn corrects_in_quotes() {
        assert_suggestion_result(
            "\"We roller skated together,\" she said.",
            RollerSkated::default(),
            "\"We roller-skated together,\" she said.",
        );
    }

    #[test]
    fn corrects_across_line_break() {
        assert_suggestion_result(
            "We\nroller skated whenever we could.",
            RollerSkated::default(),
            "We\nroller-skated whenever we could.",
        );
    }

    #[test]
    fn corrects_with_trailing_punctuation() {
        assert_suggestion_result(
            "He roller skated, laughed, and waved.",
            RollerSkated::default(),
            "He roller-skated, laughed, and waved.",
        );
    }

    #[test]
    fn corrects_without_space_after_punctuation() {
        assert_suggestion_result(
            "He roller skated,laughed, and waved.",
            RollerSkated::default(),
            "He roller-skated,laughed, and waved.",
        );
    }

    #[test]
    fn allows_hyphenated_form() {
        assert_lint_count("They roller-skated yesterday.", RollerSkated::default(), 0);
    }

    #[test]
    fn allows_subject_named_roller() {
        assert_lint_count(
            "The roller skated across the stage.",
            RollerSkated::default(),
            0,
        );
    }

    #[test]
    fn allows_other_compounds() {
        assert_lint_count(
            "Their roller skating routine impressed everyone.",
            RollerSkated::default(),
            0,
        );
    }
}
