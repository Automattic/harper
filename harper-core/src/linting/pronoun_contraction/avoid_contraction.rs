use crate::Token;
use crate::TokenKind;
use crate::expr::Expr;
use crate::expr::SequenceExpr;

use super::super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct AvoidContraction {
    expr: Box<dyn Expr>,
}

impl Default for AvoidContraction {
    fn default() -> Self {
        let pattern = SequenceExpr::aco("you're")
            .then_whitespace()
            .then_kind_is_but_is_not(TokenKind::is_nominal, TokenKind::is_likely_homograph);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for AvoidContraction {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let word = matched_tokens[0].span.get_content(source);

        Some(Lint {
            span: matched_tokens[0].span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                vec!['y', 'o', 'u', 'r'],
                word,
            )],
            message: "It appears you intended to use the possessive version of this word"
                .to_owned(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "This rule looks for situations where a contraction was used where it shouldn't have been."
    }
}
