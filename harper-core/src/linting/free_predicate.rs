use std::sync::Arc;

use crate::Token;
use crate::TokenKind;
use crate::char_string::CharStringExt;
use crate::expr::{Expr, ExprMap, SequenceExpr};
use crate::patterns::WhitespacePattern;

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct FreePredicate {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for FreePredicate {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let no_modifier = SequenceExpr::with(linking_like)
            .t_ws()
            .then(matches_fee)
            .then_optional(WhitespacePattern)
            .then(follows_fee);

        map.insert(no_modifier, 2);

        let with_adverb = SequenceExpr::with(linking_like)
            .t_ws()
            .then_adverb()
            .t_ws()
            .then(matches_fee)
            .then_optional(WhitespacePattern)
            .then(follows_fee);

        map.insert(with_adverb, 4);

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for FreePredicate {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_idx = *self.map.lookup(0, matched_tokens, source)?;
        let offending = matched_tokens.get(offending_idx)?;

        Some(Lint {
            span: offending.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "free",
                offending.span.get_content(source),
            )],
            message: "Use `free` here to show that something costs nothing.".to_owned(),
            priority: 38,
        })
    }

    fn description(&self) -> &'static str {
        "Helps swap in `free` when a linking verb is followed by the noun `fee`."
    }
}

fn matches_fee(token: &Token, source: &[char]) -> bool {
    if !token.kind.is_noun() {
        return false;
    }

    const FEE: [char; 3] = ['f', 'e', 'e'];
    let content = token.span.get_content(source);

    content.len() == FEE.len()
        && content
            .iter()
            .zip(FEE)
            .all(|(actual, expected)| actual.eq_ignore_ascii_case(&expected))
}

fn follows_fee(token: &Token, _source: &[char]) -> bool {
    if token.kind.is_hyphen() {
        return false;
    }

    token.kind.is_preposition()
        || token.kind.is_conjunction()
        || matches!(token.kind, TokenKind::Punctuation(_))
}

fn linking_like(token: &Token, source: &[char]) -> bool {
    const BE_FORMS: [&str; 8] = ["be", "is", "am", "are", "was", "were", "being", "been"];
    let content = token.span.get_content(source);

    BE_FORMS
        .iter()
        .any(|form| content.eq_ignore_ascii_case_str(form))
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    use super::FreePredicate;

    #[test]
    fn corrects_is_fee_for() {
        assert_suggestion_result(
            "The trial is fee for new members.",
            FreePredicate::default(),
            "The trial is free for new members.",
        );
    }

    #[test]
    fn corrects_totally_fee() {
        assert_suggestion_result(
            "Customer support is totally fee.",
            FreePredicate::default(),
            "Customer support is totally free.",
        );
    }

    #[test]
    fn corrects_really_fee_to() {
        assert_suggestion_result(
            "The workshop is really fee to attend.",
            FreePredicate::default(),
            "The workshop is really free to attend.",
        );
    }

    #[test]
    fn corrects_fee_with_comma() {
        assert_suggestion_result(
            "Our platform is fee, and always available.",
            FreePredicate::default(),
            "Our platform is free, and always available.",
        );
    }

    #[test]
    fn corrects_fee_period() {
        assert_suggestion_result(
            "Access is fee.",
            FreePredicate::default(),
            "Access is free.",
        );
    }

    #[test]
    fn corrects_fee_past_tense() {
        assert_suggestion_result(
            "The program was fee for nonprofits.",
            FreePredicate::default(),
            "The program was free for nonprofits.",
        );
    }

    #[test]
    fn allows_fee_based() {
        assert_no_lints("The pricing model is fee-based.", FreePredicate::default());
    }

    #[test]
    fn allows_fee_paying() {
        assert_no_lints("The membership is fee-paying.", FreePredicate::default());
    }

    #[test]
    fn allows_fee_schedule_statement() {
        assert_no_lints(
            "This plan has a fee for standard support.",
            FreePredicate::default(),
        );
    }

    #[test]
    fn allows_fee_free_phrase() {
        assert_no_lints(
            "Our service is fee-free for students.",
            FreePredicate::default(),
        );
    }

    #[test]
    fn counts_single_lint() {
        assert_lint_count(
            "The upgrade is fee for existing users.",
            FreePredicate::default(),
            1,
        );
    }
}
