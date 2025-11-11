use harper_brill::UPOS;

use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::patterns::UPOSSet;
use crate::patterns::WordSet;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind};

pub struct VerbToAdjective {
    expr: Box<dyn Expr>,
}

impl Default for VerbToAdjective {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .then(WordSet::new(&["the", "a", "an"]))
            .t_ws()
            .then(|tok: &Token, _: &[char]| {
                tok.kind.is_verb() && !tok.kind.is_adjective() && !tok.kind.is_noun()
            })
            .t_ws()
            .then(UPOSSet::new(&[UPOS::NOUN, UPOS::PROPN]));

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for VerbToAdjective {
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
}
