use harper_brill::UPOS;

use crate::linting::expr_linter::Chunk;
use crate::{
    CharStringExt, Token, TokenKind,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::UPOSSet,
};

pub(crate) struct SaleToSell {
    expr: ExprMap<usize>,
}

impl Default for SaleToSell {
    fn default() -> Self {
        let mut map = ExprMap::default();

        // "I sale clothes", "She sale cars": a subject pronoun cannot precede a noun
        // phrase, so a following noun means `sale` is being used as a verb.
        let pronoun_then_noun_follow = SequenceExpr::with(|tok: &Token, source: &[char]| {
            is_unambiguous_subject_pronoun(tok, source)
        })
        .t_ws()
        .then(|tok: &Token, source: &[char]| is_sale_word(tok, source))
        .t_ws()
        .then(UPOSSet::new(&[UPOS::NOUN]));

        map.insert(pronoun_then_noun_follow, 2);

        Self { expr: map }
    }
}

impl ExprLinter for SaleToSell {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_index = *self.expr.lookup(0, matched_tokens, source)?;
        let target = &matched_tokens[offending_index];

        Some(Lint {
            span: target.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "sell",
                target.get_ch(source),
            )],
            message: "`sale` is a noun, the verb should be `sell`.".into(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects `sale` to `sell` when a subject pronoun shows the verb is intended."
    }
}

/// Pronouns that can only be sentence subjects, unlike "you" or "it".
/// "Send you sale records" shows why the ambiguous ones must be excluded.
fn is_unambiguous_subject_pronoun(token: &Token, source: &[char]) -> bool {
    if !matches!(token.kind, TokenKind::Word(_)) {
        return false;
    }

    token
        .get_ch(source)
        .eq_any_ignore_ascii_case_str(&["he", "i", "she", "they", "we", "who"])
}

fn is_sale_word(token: &Token, source: &[char]) -> bool {
    if !matches!(token.kind, TokenKind::Word(_)) {
        return false;
    }

    token.get_ch(source).eq_any_ignore_ascii_case_str(&["sale"])
}
