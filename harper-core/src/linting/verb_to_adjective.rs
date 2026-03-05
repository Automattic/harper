use harper_brill::UPOS;

use crate::expr::All;
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::patterns::UPOSSet;
use crate::patterns::WordSet;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind};
use crate::linting::expr_linter::Chunk;

pub struct VerbToAdjective {
    expr: Box<dyn Expr>,
}

impl Default for VerbToAdjective {
    fn default() -> Self {
        let expr = SequenceExpr::word_set(&["the", "a", "an"])
            .t_ws()
            .then_kind_where(|kind| {
                (kind.is_verb()
                    && !kind.is_verb_past_form()
                    && !kind.is_adjective()
                    && !kind.is_noun())
                    || kind.is_degree_adverb()
            })
            .t_ws()
            .then(UPOSSet::new(&[UPOS::NOUN, UPOS::PROPN]));

        let exceptions = SequenceExpr::anything()
            .t_any()
            .then_unless(WordSet::new(&["very"]));

        Self {
            expr: Box::new(All::new(vec![Box::new(expr), Box::new(exceptions)])),
        }
    }
}

impl ExprLinter for VerbToAdjective {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Typo,
            suggestions: vec![],
            message: "Did you intend to use an adjective here?".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Looks for situations where a verb was written where an adjective is often intended."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_no_lints};

    use super::VerbToAdjective;

    #[test]
    fn fully_accounting() {
        assert_lint_count(
            "By scheduling time to do a fully accounting of where your CPU cycles are going, you can preemptively save yourself (and your contributors) a lot of time.",
            VerbToAdjective::default(),
            1,
        );
    }

    #[test]
    fn new_car_is_valid() {
        assert_no_lints("I really like my new car.", VerbToAdjective::default());
    }

    #[test]
    fn new_sentence_is_valid() {
        assert_no_lints(
            "I want you to write a new sentence for me.",
            VerbToAdjective::default(),
        );
    }

    #[test]
    fn correct_term_is_valid() {
        assert_no_lints(
            "Ensure the correct term is used for individuals residing abroad.",
            VerbToAdjective::default(),
        );
    }

    #[test]
    fn causes_amazement_is_valid() {
        assert_no_lints(
            "It is something that causes amazement.",
            VerbToAdjective::default(),
        );
    }

    #[test]
    fn correct_auxiliary_is_valid() {
        assert_no_lints(
            "Can you suggest a correct auxiliary?",
            VerbToAdjective::default(),
        );
    }

    #[test]
    fn submitted_form_data_is_valid() {
        assert_no_lints(
            "This is the email address that will receive the submitted form data.",
            VerbToAdjective::default(),
        );
    }

    #[test]
    fn the_unexplored_territories_is_valid() {
        assert_no_lints(
            "Not the unexplored territories, ripe for discovery, but the areas actively erased, obscured, or simply deemed unworthy of representation?",
            VerbToAdjective::default(),
        );
    }
}
