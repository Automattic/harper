use harper_brill::UPOS;

use crate::Token;
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::patterns::UPOSSet;

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct TooTo {
    expr: Box<dyn Expr>,
}

impl Default for TooTo {
    fn default() -> Self {
        let expr = SequenceExpr::aco("too").t_ws().then(UPOSSet::new(&[
            UPOS::NOUN,
            UPOS::PRON,
            UPOS::PROPN,
            UPOS::VERB,
            UPOS::DET,
        ]));
        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for TooTo {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let too_token = &matched_tokens[0];
        let span = too_token.span;
        let text = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::Typo,
            suggestions: vec![
                Suggestion::replace_with_match_case("to".chars().collect(), text)
            ],
            message: "Use the infinitive marker `to` here instead of the adverb `too`, which indicates excess degree.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Handles the transition from `too` -> `to`."
    }
}
