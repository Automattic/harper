use crate::expr::LongestMatchOf;
use crate::expr::SequenceExpr;
use crate::expr::Expr;
use crate::{
    Token, TokenStringExt,
    patterns::{Pattern, WordPatternGroup},
};

use super::{ExprLinter, Lint, LintKind};

pub struct BoringWords {
    expr: Box<dyn Expr>,
}

impl Default for BoringWords {
    fn default() -> Self {
        let mut pattern = WordPatternGroup::default();

        pattern.add_word("very");
        pattern.add_word("interesting");
        pattern.add_word("several");
        pattern.add_word("most");
        pattern.add_word("many");

        Self {
            expr: Box::new(pattern),
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
                "“{}” is a boring word. Try something a little more exotic.",
                matched_word
            ),
            priority: 127,
        })
    }

    fn description(&self) -> &'static str {
        "This rule looks for particularly boring or overused words. Using varied language is an easy way to keep a reader's attention."
    }
}
