use std::sync::Arc;

use harper_brill::UPOS;

use crate::{
    CharStringExt, Token, TokenKind,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::UPOSSet,
};

pub(super) struct AffectToEffect {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for AffectToEffect {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let word_follow = SequenceExpr::default()
            .then(|tok: &Token, _source: &[char]| is_preceding_context(tok))
            .t_ws()
            .then(|tok: &Token, source: &[char]| is_affect_word(tok, source))
            .t_ws()
            .then(UPOSSet::new(&[
                UPOS::AUX,
                UPOS::PROPN,
                UPOS::VERB,
                UPOS::INTJ,
                UPOS::ADP,
                UPOS::SCONJ,
                UPOS::ADJ,
            ]));

        map.insert(word_follow, 2);

        let punctuation_follow = SequenceExpr::default()
            .then(|tok: &Token, _source: &[char]| is_preceding_context(tok))
            .t_ws()
            .then(|tok: &Token, source: &[char]| is_affect_word(tok, source))
            .then(|tok: &Token, _source: &[char]| matches!(tok.kind, TokenKind::Punctuation(_)));

        map.insert(punctuation_follow, 2);

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for AffectToEffect {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_idx = *self.map.lookup(0, matched_tokens, source)?;
        let target = &matched_tokens[offending_idx];

        let preceding = matched_tokens[..offending_idx]
            .iter()
            .rfind(|tok| !tok.kind.is_whitespace());

        let mut following = matched_tokens[offending_idx + 1..]
            .iter()
            .filter(|tok| !tok.kind.is_whitespace());

        let first_following = following.next();

        if matched_tokens
            .first()
            .is_some_and(|tok| tok.kind.is_possessive_nominal())
        {
            return None;
        }

        if let Some(prev) = preceding {
            if prev.kind.is_upos(UPOS::AUX) || prev.kind.is_upos(UPOS::VERB) {
                let lower_prev = prev.span.get_content_string(source).to_lowercase();

                if !matches!(
                    lower_prev.as_str(),
                    "take" | "takes" | "taking" | "took" | "taken"
                ) {
                    return None;
                }
            }
        }

        if first_following
            .is_some_and(|tok| tok.kind.is_upos(UPOS::AUX) || tok.kind.is_upos(UPOS::VERB))
        {
            if preceding
                .is_some_and(|tok| tok.kind.is_upos(UPOS::AUX) || tok.kind.is_upos(UPOS::VERB))
            {
                return None;
            }
        }

        let token_text = target.span.get_content_string(source);
        let lower = token_text.to_lowercase();
        let replacement = match lower.as_str() {
            "affect" => "effect",
            "affects" => "effects",
            _ => return None,
        };

        Some(Lint {
            span: target.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                replacement,
                target.span.get_content(source),
            )],
            message: "`affect` is usually a verb; use `effect` here for the result or outcome."
                .into(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects `affect` to `effect` when the context shows the noun meaning `result`."
    }
}

fn is_affect_word(token: &Token, source: &[char]) -> bool {
    const AFFECT: &[char] = &['a', 'f', 'f', 'e', 'c', 't'];
    const AFFECTS: &[char] = &['a', 'f', 'f', 'e', 'c', 't', 's'];

    if !matches!(token.kind, TokenKind::Word(_)) {
        return false;
    }

    let text = token.span.get_content(source);
    text.eq_ignore_ascii_case_chars(AFFECT) || text.eq_ignore_ascii_case_chars(AFFECTS)
}

fn is_preceding_context(token: &Token) -> bool {
    if token.kind.is_adverb() {
        return false;
    }

    matches!(token.kind, TokenKind::Punctuation(_))
        || token.kind.is_preposition()
        || token.kind.is_conjunction()
        || token.kind.is_proper_noun()
        || token.kind.is_verb()
        || token.kind.is_adjective()
        || token.kind.is_determiner()
        || token.kind.is_noun()
}
