use std::sync::Arc;

use harper_brill::UPOS;

use crate::{
    CharStringExt, Token, TokenKind,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::{UPOSSet, WhitespacePattern},
};

pub struct EffectAffect {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for EffectAffect {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let context = SequenceExpr::default()
            .then(UPOSSet::new(&[UPOS::PART, UPOS::PUNCT, UPOS::NOUN]))
            .t_ws()
            .then(|tok: &Token, source: &[char]| is_effect_word(tok, source))
            .t_ws()
            .then(UPOSSet::new(&[
                UPOS::ADV,
                UPOS::AUX,
                UPOS::PRON,
                UPOS::PROPN,
                UPOS::VERB,
                UPOS::NUM,
                UPOS::NOUN,
                UPOS::INTJ,
                UPOS::SCONJ,
            ]))
            .then_optional(WhitespacePattern)
            .then_optional(UPOSSet::new(&[UPOS::NOUN, UPOS::PUNCT]))
            .then_optional(WhitespacePattern);

        map.insert(context, 2);

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for EffectAffect {
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

        let first_following = following.next()?;
        let second_following = following.next();

        // Avoid "to effect change", which uses the legitimate verb "effect".
        if let Some(prev) = preceding {
            if is_token_to(prev, source) && is_change_like(first_following, source) {
                return None;
            }
        }

        // Skip when the context already shows a clear noun usage (e.g., "the effect your idea had").
        if let Some(prev) = preceding {
            if prev.kind.is_determiner() || prev.kind.is_adjective() {
                return None;
            }
        }

        // Do not flag when the following noun is clearly the result of "effect" in the idiomatic sense.
        if let Some(next) = second_following {
            if next.kind.is_noun() && is_change_like(next, source) {
                return None;
            }
        }

        let token_text = target.span.get_content_string(source);
        let lower = token_text.to_lowercase();
        let replacement = match lower.as_str() {
            "effect" => "affect",
            "effects" => "affects",
            _ => return None,
        };

        Some(Lint {
            span: target.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                replacement,
                target.span.get_content(source),
            )],
            message:
                "Use `affect` for the verb meaning to influence; `effect` usually names the result."
                    .into(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects `effect` to `affect` when the context shows the verb meaning `influence`."
    }
}

fn is_effect_word(token: &Token, source: &[char]) -> bool {
    if !matches!(token.kind, TokenKind::Word(_)) {
        return false;
    }

    const EFFECT: &[char] = &['e', 'f', 'f', 'e', 'c', 't'];
    const EFFECTS: &[char] = &['e', 'f', 'f', 'e', 'c', 't', 's'];

    let text = token.span.get_content(source);
    text.eq_ignore_ascii_case_chars(EFFECT) || text.eq_ignore_ascii_case_chars(EFFECTS)
}

fn is_token_to(token: &Token, source: &[char]) -> bool {
    token
        .span
        .get_content(source)
        .eq_ignore_ascii_case_chars(&['t', 'o'])
}

fn is_change_like(token: &Token, source: &[char]) -> bool {
    if !token.kind.is_noun() {
        return false;
    }

    matches!(
        token
            .span
            .get_content_string(source)
            .to_lowercase()
            .as_str(),
        "change" | "changes"
    )
}

#[cfg(test)]
mod tests {
    use super::EffectAffect;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_noun_subject_effects_object() {
        assert_suggestion_result(
            "System outages effect our customers.",
            EffectAffect::default(),
            "System outages affect our customers.",
        );
    }

    #[test]
    fn corrects_effects_variant() {
        assert_suggestion_result(
            "This policy effects employee morale.",
            EffectAffect::default(),
            "This policy affects employee morale.",
        );
    }

    #[test]
    fn ignores_effect_change_idiom() {
        assert_lint_count(
            "Leaders work to effect change in their communities.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn ignores_effect_noun_phrase() {
        assert_lint_count(
            "The effect your plan had was dramatic.",
            EffectAffect::default(),
            0,
        );
    }
}
