use harper_brill::UPOS;

use crate::expr::{Expr, OwnedExprExt, SequenceExpr};
use crate::patterns::{UPOSSet, WordSet};
use crate::{Span, Token};

use super::{ExprLinter, Lint, LintKind, Suggestion};

const AMBIGUOUS_ADVERBS: &[&str] = &["just", "not"];

pub struct ToAdverb {
    expr: Box<dyn Expr>,
}

impl Default for ToAdverb {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then(UPOSSet::new(&[UPOS::ADV]).or(WordSet::new(AMBIGUOUS_ADVERBS)))
            .t_ws()
            .t_aco("to")
            .t_ws()
            .then_verb();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ToAdverb {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        let first_to = tokens.first()?;
        let second_to_idx = 4;
        let second_to = tokens.get(second_to_idx)?;

        let adverb_idx = 2;

        let adverb = tokens.get(adverb_idx)?;

        let span = Span::new(first_to.span.start, second_to.span.end);
        let keep_first_variant = source[first_to.span.start..adverb.span.end].to_vec();
        let drop_first_variant = source[adverb.span.start..second_to.span.end].to_vec();

        if keep_first_variant.is_empty() || drop_first_variant.is_empty() {
            return None;
        }

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![
                Suggestion::ReplaceWith(keep_first_variant),
                Suggestion::ReplaceWith(drop_first_variant),
            ],
            message: "Remove the repeated `to` in this infinitive.".to_owned(),
            priority: 40,
        })
    }

    fn description(&self) -> &'static str {
        "Flags duplicated `to` around certain adverbs (e.g. `to never to`) and offers fixes that keep only one `to`."
    }
}

#[cfg(test)]
mod tests {
    use super::ToAdverb;
    use crate::linting::tests::{
        assert_lint_count, assert_nth_suggestion_result, assert_suggestion_count,
        assert_suggestion_result,
    };

    #[test]
    fn corrects_to_never_to() {
        assert_suggestion_result(
            "Tom has decided to never to do that again.",
            ToAdverb::default(),
            "Tom has decided to never do that again.",
        );
    }

    #[test]
    fn alternative_moves_adverb() {
        assert_nth_suggestion_result(
            "Tom has decided to never to do that again.",
            ToAdverb::default(),
            "Tom has decided never to do that again.",
            1,
        );
    }

    #[test]
    fn corrects_to_maybe_to() {
        assert_suggestion_result(
            "The next step is to maybe to take a language class.",
            ToAdverb::default(),
            "The next step is to maybe take a language class.",
        );
    }

    #[test]
    fn corrects_to_not_to() {
        assert_suggestion_result(
            "He tells the monitor to not to collect anything.",
            ToAdverb::default(),
            "He tells the monitor to not collect anything.",
        );
    }

    #[test]
    fn corrects_to_just_to() {
        assert_suggestion_result(
            "She told me to just to keep the peace.",
            ToAdverb::default(),
            "She told me to just keep the peace.",
        );
    }

    #[test]
    fn corrects_to_really_to() {
        assert_suggestion_result(
            "They plan to really to push the release.",
            ToAdverb::default(),
            "They plan to really push the release.",
        );
    }

    #[test]
    fn offers_two_suggestions() {
        assert_suggestion_count(
            "He agreed to probably to lead the effort.",
            ToAdverb::default(),
            2,
        );
    }

    #[test]
    fn allows_single_to_with_adverb() {
        assert_lint_count("He wants to always win the match.", ToAdverb::default(), 0);
    }

    #[test]
    fn corrects_to_quickly_to() {
        assert_suggestion_result(
            "They hoped to quickly to solve it.",
            ToAdverb::default(),
            "They hoped to quickly solve it.",
        );
    }

    #[test]
    fn ignores_missing_verb_after_second_to() {
        assert_lint_count("We tried to eventually to.", ToAdverb::default(), 0);
    }

    #[test]
    fn handles_capitalized_to() {
        assert_suggestion_result(
            "To Always to succeed is the goal.",
            ToAdverb::default(),
            "To Always succeed is the goal.",
        );
    }
}
