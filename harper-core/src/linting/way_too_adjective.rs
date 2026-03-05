use harper_brill::UPOS;

use crate::Token;
use crate::expr::{All, Expr, OwnedExprExt, SequenceExpr};
use crate::patterns::{UPOSSet, WordSet};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct WayTooAdjective {
    expr: Box<dyn Expr>,
}

impl Default for WayTooAdjective {
    fn default() -> Self {
        let base = SequenceExpr::default()
            .t_aco("way")
            .t_ws()
            .t_aco("to")
            .t_ws()
            .then(UPOSSet::new(&[UPOS::ADJ]).or(WordSet::new(&["much"])));

        let exceptions = SequenceExpr::anything()
            .t_any()
            .t_any()
            .t_any()
            .then_word_set(&["surface", "return", "aqua"]);

        let expr = All::new(vec![
            Box::new(base),
            Box::new(SequenceExpr::unless(exceptions)),
        ]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for WayTooAdjective {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let to_tok = toks.get(2)?;
        let span = to_tok.span;
        let original = span.get_content(src);

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case(
                "too".chars().collect(),
                original,
            )],
            message: "Did you mean “too”?".into(),
            priority: 25,
        })
    }

    fn description(&self) -> &'static str {
        "Replaces the preposition `to` with the adverb `too` after `way` when followed by an \
         adjective (e.g. `way too fast`)"
    }
}

#[cfg(test)]
mod tests {
    use super::WayTooAdjective;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_way_to_fast() {
        assert_suggestion_result(
            "You drive way to fast.",
            WayTooAdjective::default(),
            "You drive way too fast.",
        );
    }

    #[test]
    fn corrects_way_to_complicated() {
        assert_suggestion_result(
            "I think this would be way to complicated to implement.",
            WayTooAdjective::default(),
            "I think this would be way too complicated to implement.",
        );
    }

    #[test]
    fn corrects_way_to_much() {
        assert_suggestion_result(
            "…and ate way to much.",
            WayTooAdjective::default(),
            "…and ate way too much.",
        );
    }

    #[test]
    fn allows_fast_way_to_test() {
        assert_lint_count(
            "Fast way to test daily defence teams?",
            WayTooAdjective::default(),
            0,
        );
    }
}
