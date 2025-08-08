use harper_brill::UPOS;

use crate::Token;
use crate::expr::AnchorEnd;
use crate::expr::Expr;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::patterns::AnyPattern;
use crate::patterns::UPOSSet;

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ToToo {
    expr: Box<dyn Expr>,
}

impl Default for ToToo {
    fn default() -> Self {
        let most = SequenceExpr::aco("to")
            .t_ws()
            .then(UPOSSet::new(&[UPOS::ADJ, UPOS::ADV]))
            .then_optional(AnyPattern)
            .then_optional(AnyPattern)
            .then_optional(AnyPattern)
            .then_optional(AnyPattern)
            .then_optional(AnyPattern)
            .then_optional(AnyPattern)
            .or(SequenceExpr::default()
                .t_aco("to")
                .then(AnchorEnd)
                .then(|tok: &Token, _: &[char]| tok.kind.is_sentence_terminator()));

        Self {
            expr: Box::new(most),
        }
    }
}

impl ExprLinter for ToToo {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let to_tok = matched_tokens.first()?;
        let span = to_tok.span;
        let original = span.get_content(source);

        for i in 2..matched_tokens.len() {
            if let Some(tok) = matched_tokens.get(i) {
                if tok
                    .kind
                    .as_word()
                    .and_then(|m| Some(m.as_ref()?.pos_tag))
                    .is_some_and(|tag| tag == Some(UPOS::ADP))
                {
                    break;
                }
                if tok.kind.is_np_member() {
                    return None;
                }
            }
        }

        if let Some(tok) = matched_tokens.get(4)
            && let Some(Some(meta)) = tok.kind.as_word()
            && let Some(tag) = meta.pos_tag
            && tag.is_nominal()
        {
            return None;
        }

        Some(Lint {
            span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case(
                "too".chars().collect(),
                original,
            )],
            message: "Use `too` (with two o’s) when indicating excess or addition.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Handles the transition from `to` -> `too`."
    }
}
