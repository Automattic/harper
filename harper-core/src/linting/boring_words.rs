use crate::expr::{Expr, WordExprGroup};
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind};

pub struct BoringWords {
    expr: Box<dyn Expr>,
}

impl Default for BoringWords {
    fn default() -> Self {
        let mut expr = WordExprGroup::default();

        expr.add_word("very");
        expr.add_word("interesting");
        expr.add_word("several");
        expr.add_word("most");
        expr.add_word("many");

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for BoringWords {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let matched_word = matched_tokens.span()?.get_content_string(source);

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Enhancement,
            suggestions: vec![],
            message: format!(
                "“{matched_word}” is a boring word. Try something a little more exotic."
            ),
            priority: 127,
        })
    }

    fn description(&self) -> &'static str {
        "This rule looks for particularly boring or overused words. Using varied language is an easy way to keep a reader's attention."
    }
}
