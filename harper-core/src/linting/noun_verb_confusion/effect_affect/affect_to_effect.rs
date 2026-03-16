use std::sync::Arc;

use harper_brill::UPOS;

use crate::linting::expr_linter::Chunk;
use crate::{
    CharStringExt, Token, TokenKind,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::{ModalVerb, Pattern, UPOSSet},
};

pub(super) struct AffectToEffect {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for AffectToEffect {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let adj_then_noun_follow = SequenceExpr::with(|tok: &Token, source: &[char]| {
            matches_preceding_context_adj_noun(tok, source)
        })
        .t_ws()
        .then(|tok: &Token, source: &[char]| is_affect_word(tok, source))
        .t_ws()
        .then(UPOSSet::new(&[UPOS::ADJ]))
        .t_ws()
        .then(UPOSSet::new(&[UPOS::NOUN]));

        map.insert(adj_then_noun_follow, 2);

        let word_follow = SequenceExpr::with(|tok: &Token, source: &[char]| {
            matches_preceding_context(tok, source)
        })
        .t_ws()
        .then(|tok: &Token, source: &[char]| is_affect_word(tok, source))
        .t_ws()
        .then(UPOSSet::new(&[
            UPOS::PROPN,
            UPOS::INTJ,
            UPOS::ADP,
            UPOS::SCONJ,
        ]));

        map.insert(word_follow, 2);

        let verb_follow = SequenceExpr::with(|tok: &Token, source: &[char]| {
            matches_preceding_context_verb_follow(tok, source)
        })
        .t_ws()
        .then(|tok: &Token, source: &[char]| is_affect_word(tok, source))
        .t_ws()
        .then(UPOSSet::new(&[UPOS::AUX, UPOS::VERB]));

        map.insert(verb_follow, 2);

        let punctuation_follow = SequenceExpr::with(|tok: &Token, source: &[char]| {
            matches_preceding_context(tok, source)
        })
        .t_ws()
        .then(|tok: &Token, source: &[char]| is_affect_word(tok, source))
        .then_kind_where(|kind| kind.is_punctuation());

        map.insert(punctuation_follow, 2);

        let great_affect = SequenceExpr::default()
            .t_aco("great")
            .t_ws()
            .then(|tok: &Token, source: &[char]| is_affect_word(tok, source));

        map.insert(great_affect, 2);

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for AffectToEffect {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_index = *self.map.lookup(0, matched_tokens, source)?;
        let target = &matched_tokens[offending_index];

        let preceding = matched_tokens[..offending_index]
            .iter()
            .rfind(|tok| !tok.kind.is_whitespace());

        if preceding.is_some_and(|tok| {
            (tok.kind.is_pronoun() || tok.kind.is_upos(UPOS::PRON))
                && !tok.kind.is_possessive_pronoun()
        }) {
            // Pronouns like "it" or "they" almost always introduce the verb form ("it affects").
            return None;
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

fn is_take_form(chars: &[char]) -> bool {
    chars.eq_ignore_ascii_case_str("take")
        || chars.eq_ignore_ascii_case_str("takes")
        || chars.eq_ignore_ascii_case_str("taking")
        || chars.eq_ignore_ascii_case_str("took")
        || chars.eq_ignore_ascii_case_str("taken")
}

fn is_modal_like(token: &Token, source: &[char], prev: &[char]) -> bool {
    if ModalVerb::default()
        .matches(std::slice::from_ref(token), source)
        .is_some()
    {
        return true;
    }

    prev.eq_ignore_ascii_case_str("do")
        || prev.eq_ignore_ascii_case_str("does")
        || prev.eq_ignore_ascii_case_str("did")
        || prev.eq_ignore_ascii_case_str("don't")
        || prev.eq_ignore_ascii_case_str("dont")
        || prev.eq_ignore_ascii_case_str("doesn't")
        || prev.eq_ignore_ascii_case_str("doesnt")
        || prev.eq_ignore_ascii_case_str("didn't")
        || prev.eq_ignore_ascii_case_str("didnt")
}

fn matches_preceding_context(token: &Token, source: &[char]) -> bool {
    matches_preceding_context_impl(token, source, true, true)
}

fn matches_preceding_context_adj_noun(token: &Token, source: &[char]) -> bool {
    matches_preceding_context_impl(token, source, false, true)
}

fn matches_preceding_context_verb_follow(token: &Token, source: &[char]) -> bool {
    matches_preceding_context_impl(token, source, true, false)
}

fn matches_preceding_context_impl(
    token: &Token,
    source: &[char],
    allow_noun_like: bool,
    allow_verb_like: bool,
) -> bool {
    if token.kind.is_possessive_nominal() {
        return false;
    }

    if !is_preceding_context(token) {
        return false;
    }

    let content = token.span.get_content(source);
    let is_take_form_word = is_take_form(content);

    if behaves_like_verb(token, source, content) && !is_take_form_word {
        return false;
    }

    if !allow_verb_like && token.kind.is_upos(UPOS::VERB) && !is_take_form_word {
        return false;
    }

    if !allow_noun_like
        && (token.kind.is_noun() || token.kind.is_proper_noun())
        && !is_take_form_word
    {
        return false;
    }

    true
}

fn behaves_like_verb(token: &Token, source: &[char], prev: &[char]) -> bool {
    token.kind.is_upos(UPOS::AUX)
        || token.kind.is_auxiliary_verb()
        || is_modal_like(token, source, prev)
}

fn is_preceding_context(token: &Token) -> bool {
    if token.kind.is_upos(UPOS::ADV) {
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
