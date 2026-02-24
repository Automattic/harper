use std::sync::Arc;

use crate::linting::expr_linter::Chunk;
use crate::{
    CharStringExt, Token,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::ModalVerb,
};

#[derive(Clone, Copy, Default)]
struct MatchContext {
    modal_index: usize,
}

pub struct ModalSeem {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<MatchContext>>,
}

impl ModalSeem {
    fn base_sequence() -> SequenceExpr {
        SequenceExpr::with(ModalVerb::default())
            .t_ws()
            .t_aco("seen")
    }

    fn adjective_step() -> SequenceExpr {
        SequenceExpr::default()
            .t_ws()
            .then_kind_where(|kind| kind.is_adjective())
    }

    fn adverb_then_adjective_step() -> SequenceExpr {
        SequenceExpr::default()
            .t_ws()
            .then_kind_where(|kind| kind.is_adverb())
            .t_ws()
            .then_kind_where(|kind| kind.is_adjective())
    }
}

impl Default for ModalSeem {
    fn default() -> Self {
        let mut map = ExprMap::default();

        map.insert(
            SequenceExpr::default()
                .then_seq(Self::base_sequence())
                .then(Self::adjective_step()),
            MatchContext::default(),
        );

        map.insert(
            SequenceExpr::default()
                .then_seq(Self::base_sequence())
                .then(Self::adverb_then_adjective_step()),
            MatchContext::default(),
        );

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for ModalSeem {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let context = self.map.lookup(0, matched_tokens, source)?;

        let seen_token = matched_tokens
            .iter()
            .skip(context.modal_index)
            .find(|tok| {
                tok.span
                    .get_content(source)
                    .eq_ignore_ascii_case_str("seen")
            })?;

        let span = seen_token.span;
        let original = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![
                Suggestion::replace_with_match_case("seem".chars().collect(), original),
                Suggestion::replace_with_match_case("be".chars().collect(), original),
            ],
            message: "Swap `seen` for a linking verb when it follows a modal before an adjective."
                .to_owned(),
            priority: 32,
        })
    }

    fn description(&self) -> &str {
        "Detects modal verbs followed by `seen` before adjectives and suggests `seem` or `be`."
    }
}

#[cfg(test)]
mod tests {
    use super::ModalSeem;
    use crate::linting::tests::{
        assert_lint_count, assert_no_lints, assert_nth_suggestion_result, assert_suggestion_result,
    };

    #[test]
    fn corrects_basic_case() {
        assert_suggestion_result(
            "It may seen impossible to finish.",
            ModalSeem::default(),
            "It may seem impossible to finish.",
        );
    }

    #[test]
    fn corrects_with_adverb() {
        assert_suggestion_result(
            "That might seen utterly ridiculous.",
            ModalSeem::default(),
            "That might seem utterly ridiculous.",
        );
    }

    #[test]
    fn offers_be_option() {
        assert_nth_suggestion_result(
            "It may seen impossible to finish.",
            ModalSeem::default(),
            "It may be impossible to finish.",
            1,
        );
    }

    #[test]
    fn respects_uppercase() {
        assert_suggestion_result(
            "THIS COULD SEEN TERRIBLE.",
            ModalSeem::default(),
            "THIS COULD SEEM TERRIBLE.",
        );
    }

    #[test]
    fn corrects_before_punctuation() {
        assert_suggestion_result(
            "Still, it may seen absurd, but we will continue.",
            ModalSeem::default(),
            "Still, it may seem absurd, but we will continue.",
        );
    }

    #[test]
    fn corrects_across_newline() {
        assert_suggestion_result(
            "It may seen\n impossible to pull off.",
            ModalSeem::default(),
            "It may seem\n impossible to pull off.",
        );
    }

    #[test]
    fn ignores_correct_seem() {
        assert_no_lints("It may seem impossible to finish.", ModalSeem::default());
    }

    #[test]
    fn ignores_modal_with_be_seen() {
        assert_no_lints("It may be seen as unfair.", ModalSeem::default());
    }

    #[test]
    fn ignores_modal_seen_noun() {
        assert_no_lints(
            "It may seen results sooner than expected.",
            ModalSeem::default(),
        );
    }

    #[test]
    fn ignores_modal_seen_clause() {
        assert_lint_count(
            "It may seen that we are improving.",
            ModalSeem::default(),
            0,
        );
    }
}
