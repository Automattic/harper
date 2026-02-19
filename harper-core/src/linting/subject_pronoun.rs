use crate::expr::{AnchorStart, Expr, SequenceExpr};
use crate::{Token, TokenStringExt};

use super::expr_linter::Chunk;
use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct SubjectPronoun {
    expr: Box<dyn Expr>,
}

impl Default for SubjectPronoun {
    fn default() -> Self {
        let expr = SequenceExpr::with(AnchorStart)
            .t_aco("me")
            .t_ws()
            .t_aco("and")
            .t_ws()
            .then_proper_noun();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for SubjectPronoun {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;

        let mut suggestion_chars = Vec::new();
        suggestion_chars.extend_from_slice(matched_tokens.last()?.span.get_content(source));
        suggestion_chars.extend(" and I".chars());

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::ReplaceWith(suggestion_chars)],
            message: "Put the other person first and use `I` in this compound subject.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Fixes sentences that start with `me and X` by putting the proper noun first and using `I`."
    }
}

fn append_token_chars(chars: &mut Vec<char>, token: &Token, source: &[char]) {
    chars.extend(token.span.get_content(source).iter().copied());
}

fn append_tokens_chars(chars: &mut Vec<char>, tokens: &[Token], source: &[char]) {
    for token in tokens {
        append_token_chars(chars, token, source);
    }
}

#[cfg(test)]
mod tests {
    use super::SubjectPronoun;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn alex_ladder() {
        assert_suggestion_result(
            "Me and Alex carried the huge ladder.",
            SubjectPronoun::default(),
            "Alex and I carried the huge ladder.",
        );
    }

    #[test]
    fn jordan_lamp() {
        assert_suggestion_result(
            "Me and Jordan fixed the broken lamp.",
            SubjectPronoun::default(),
            "Jordan and I fixed the broken lamp.",
        );
    }

    #[test]
    fn taylor_crate() {
        assert_suggestion_result(
            "Me and Taylor opened the dusty crate.",
            SubjectPronoun::default(),
            "Taylor and I opened the dusty crate.",
        );
    }

    #[test]
    fn kayla_dog() {
        assert_suggestion_result(
            "Me and Kayla chased the noisy dog.",
            SubjectPronoun::default(),
            "Kayla and I chased the noisy dog.",
        );
    }

    #[test]
    fn madison_yard() {
        assert_suggestion_result(
            "Me and Madison painted the small yard shed.",
            SubjectPronoun::default(),
            "Madison and I painted the small yard shed.",
        );
    }

    #[test]
    fn avery_tree() {
        assert_suggestion_result(
            "Me and Avery climbed the old tree.",
            SubjectPronoun::default(),
            "Avery and I climbed the old tree.",
        );
    }

    #[test]
    fn blake_room() {
        assert_suggestion_result(
            "Me and Blake cleaned the crowded room.",
            SubjectPronoun::default(),
            "Blake and I cleaned the crowded room.",
        );
    }

    #[test]
    fn riley_train() {
        assert_suggestion_result(
            "Me and Riley watched the slow train go by.",
            SubjectPronoun::default(),
            "Riley and I watched the slow train go by.",
        );
    }

    #[test]
    fn cameron_door() {
        assert_suggestion_result(
            "Me and Cameron fixed the loose door hinge.",
            SubjectPronoun::default(),
            "Cameron and I fixed the loose door hinge.",
        );
    }

    #[test]
    fn jamie_bag() {
        assert_suggestion_result(
            "Me and Jamie carried the heavy shopping bag.",
            SubjectPronoun::default(),
            "Jamie and I carried the heavy shopping bag.",
        );
    }
}
