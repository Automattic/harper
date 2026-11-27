use super::super::conjugate_pair_verb;

use crate::linting::expr_linter::Chunk;
use crate::{
    CharStringExt, Token, TokenKind,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub(crate) struct SaleToSell {
    expr: ExprMap<usize>,
}

impl Default for SaleToSell {
    fn default() -> Self {
        let mut map = ExprMap::default();

        // "I sale clothes", "She sale cars": a subject pronoun cannot precede a noun
        // phrase, so a following noun means `sale` is being used as a verb.
        let pronoun_then_noun_follow =
            SequenceExpr::with(|tok: &Token, source: &[char]| is_subject_pronoun(tok, source))
                .t_ws()
                .then(|tok: &Token, source: &[char]| is_sale_word(tok, source))
                .t_ws()
                // Match the dictionary kind rather than the context tag:
                // "fast" is tagged as an adjective in "It sale fast."
                .then_noun();

        map.insert(pronoun_then_noun_follow, 2);

        Self { expr: map }
    }
}

impl ExprLinter for SaleToSell {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // "It" is only a subject at the start of a clause.
        if matched_tokens[0].get_ch(source).eq_str("it")
            && ctx.is_some_and(|(before, _)| before.iter().any(|t| !t.kind.is_whitespace()))
        {
            return None;
        }

        let offending_index = *self.expr.lookup(0, matched_tokens, source)?;
        let target = &matched_tokens[offending_index];

        let verb = conjugate_pair_verb("sell", &matched_tokens[0], source);

        Some(Lint {
            span: target.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                &verb,
                target.get_ch(source),
            )],
            message: format!("`sale` is a noun, the verb should be `{verb}`."),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects `sale` to the correctly conjugated `sell`/`sells` when a subject pronoun shows the verb is intended."
    }
}

/// Pronouns that can act as sentence subjects; see `match_to_lint_with_context`
/// for how "it" is handled.
fn is_subject_pronoun(token: &Token, source: &[char]) -> bool {
    if !matches!(token.kind, TokenKind::Word(_)) {
        return false;
    }

    token
        .get_ch(source)
        .eq_any_ignore_ascii_case_str(&["he", "i", "it", "she", "they", "we", "who"])
}

fn is_sale_word(token: &Token, source: &[char]) -> bool {
    if !matches!(token.kind, TokenKind::Word(_)) {
        return false;
    }

    token.get_ch(source).eq_any_ignore_ascii_case_str(&["sale"])
}
