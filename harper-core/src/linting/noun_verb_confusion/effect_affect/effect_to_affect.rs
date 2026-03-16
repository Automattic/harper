use std::sync::Arc;

use harper_brill::UPOS;

use crate::linting::expr_linter::Chunk;
use crate::{
    CharStringExt, Token, TokenKind,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WhitespacePattern,
};

pub(super) struct EffectToAffect {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for EffectToAffect {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let context = SequenceExpr::with(matches_preceding_context)
            .t_ws()
            .then(|tok: &Token, source: &[char]| is_effect_word(tok, source))
            .t_ws()
            .then(matches_following_context)
            .then_optional(WhitespacePattern)
            .then_optional(matches_optional_following)
            .then_optional(WhitespacePattern);

        map.insert(context, 2);

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for EffectToAffect {
    type Unit = Chunk;

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

        if let Some(prev) = preceding {
            let lower_prev = prev.span.get_content_string(source).to_lowercase();

            if matches!(
                lower_prev.as_str(),
                "take" | "takes" | "taking" | "took" | "taken"
            ) {
                return None;
            }
        }

        if first_following.kind.is_upos(UPOS::AUX) || first_following.kind.is_linking_verb() {
            return None;
        }

        let first_following_lower = first_following
            .span
            .get_content_string(source)
            .to_lowercase();

        if matches!(
            first_following_lower.as_str(),
            "is" | "are" | "was" | "were" | "be" | "been" | "being"
        ) {
            return None;
        }

        // Avoid "to effect change", which uses the legitimate verb "effect".
        if let Some(prev) = preceding
            && is_token_to(prev, source)
            && is_change_like(first_following, source)
        {
            return None;
        }

        if first_following.kind.is_upos(UPOS::VERB)
            && preceding.is_some_and(|tok| {
                tok.kind.is_upos(UPOS::NOUN)
                    || tok.kind.is_upos(UPOS::DET)
                    || tok.kind.is_upos(UPOS::ADJ)
                    || (tok.kind.is_noun()
                        && !tok.kind.is_upos(UPOS::VERB)
                        && !tok.kind.is_upos(UPOS::AUX))
            })
        {
            return None;
        }

        // Skip when the context already shows a clear noun usage (e.g., "the effect your idea had").
        if let Some(prev) = preceding
            && (prev.kind.is_upos(UPOS::DET) || prev.kind.is_upos(UPOS::ADJ))
        {
            return None;
        }

        // Do not flag when the following noun is clearly the result of "effect" in the idiomatic sense.
        if let Some(next) = second_following
            && next.kind.is_noun()
            && is_change_like(next, source)
        {
            return None;
        }

        let token_text = target.span.get_content_string(source);
        let lower = token_text.to_lowercase();

        if lower.as_str() == "effects" && preceding.is_some_and(|tok| tok.kind.is_upos(UPOS::VERB))
        {
            // Imperative phrases like "Avoid effects" legitimately use the noun.
            return None;
        }

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
    if !token.kind.is_word() {
        return false;
    }

    matches!(
        token
            .span
            .get_content_string(source)
            .to_lowercase()
            .as_str(),
        "change" | "changes" | "substitution" | "substitutions"
    )
}

fn matches_preceding_context(token: &Token, _source: &[char]) -> bool {
    tag_matches_any(
        token,
        &[
            UPOS::PART,
            UPOS::NOUN,
            UPOS::PRON,
            UPOS::PROPN,
            UPOS::ADV,
            UPOS::AUX,
            UPOS::VERB,
            UPOS::ADJ,
        ],
    )
}

fn matches_following_context(token: &Token, _source: &[char]) -> bool {
    tag_matches_any(
        token,
        &[
            UPOS::ADV,
            UPOS::AUX,
            UPOS::PRON,
            UPOS::PROPN,
            UPOS::VERB,
            UPOS::NUM,
            UPOS::NOUN,
            UPOS::INTJ,
            UPOS::SCONJ,
            UPOS::DET,
            UPOS::ADJ,
        ],
    )
}

fn matches_optional_following(token: &Token, _source: &[char]) -> bool {
    if token.kind.is_punctuation() {
        return true;
    }

    tag_matches_any(token, &[UPOS::NOUN])
}

fn tag_matches_any(token: &Token, allowed: &[UPOS]) -> bool {
    let Some(word_meta_opt) = token.kind.as_word() else {
        return false;
    };

    match word_meta_opt {
        Some(meta) => meta.pos_tag.is_none_or(|tag| allowed.contains(&tag)),
        None => true,
    }
}
