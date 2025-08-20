use std::sync::Arc;

use harper_brill::UPOS;
use serde::de::SeqAccess;

use crate::Token;
use crate::expr::All;
use crate::expr::Expr;
use crate::expr::ExprMap;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::patterns::UPOSSet;

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ToToo {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for ToToo {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let a = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then(UPOSSet::new(&[
                UPOS::ADV,
                UPOS::PUNCT,
                UPOS::ADP,
                UPOS::VERB,
                UPOS::SYM,
                UPOS::PART,
                UPOS::ADJ,
            ]));

        map.insert(a, 1);

        let b = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then(UPOSSet::new(&[
                UPOS::SCONJ,
                UPOS::PUNCT,
                UPOS::PART,
                UPOS::PROPN,
                UPOS::ADV,
                UPOS::ADJ,
                UPOS::ADP,
            ]));

        map.insert(b, 1);

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for ToToo {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let idx = self.map.lookup(0, matched_tokens, source)?;

        let to_tok = &matched_tokens[*idx];

        if !(to_tok.kind.is_upos(UPOS::ADP) || to_tok.kind.is_upos(UPOS::PART)) {
            return None;
        }

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
                if tok.kind.is_np_member() || tok.kind.is_unlintable() {
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
            message: "Use `too` (with two `o`’s) when indicating excess or addition.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Handles the transition from `to` -> `too`."
    }
}
