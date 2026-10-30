use crate::{
    Lint, Token, TokenKind, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct SeemsLikeAdjective {
    expr: SequenceExpr,
}

impl Default for SeemsLikeAdjective {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set([
                "appear", "appears", "feel", "feels", "look", "looks", "seem", "seems", "smell",
                "smells", "sound", "sounds", "taste", "tastes",
            ])
            .t_ws()
            .t_aco("like")
            .t_ws()
            .then_kind_is_but_is_not(TokenKind::is_adjective, TokenKind::is_noun),
        }
    }
}

impl ExprLinter for SeemsLikeAdjective {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));
        let span = toks[1..=2].span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::Remove],
            message: "Verbs of perception should not be followed by 'like'.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects verbs of perception redundantly followed by `like` when used with an adjective."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::SeemsLikeAdjective;

    #[test]
    fn appears() {
        assert_suggestion_result(
            "appears like awesome",
            SeemsLikeAdjective::default(),
            "appears awesome",
        );
    }

    #[test]
    fn feels() {
        assert_suggestion_result(
            "feels like lumpy",
            SeemsLikeAdjective::default(),
            "feels lumpy",
        );
    }

    #[test]
    fn looks() {
        assert_suggestion_result(
            "looks like handsome",
            SeemsLikeAdjective::default(),
            "looks handsome",
        );
    }

    #[test]
    fn seems() {
        assert_suggestion_result(
            "seems like shabby",
            SeemsLikeAdjective::default(),
            "seems shabby",
        );
    }

    #[test]
    fn sounds() {
        assert_suggestion_result(
            "sounds like loud",
            SeemsLikeAdjective::default(),
            "sounds loud",
        );
    }

    #[test]
    fn smells() {
        assert_suggestion_result(
            "smells like unpleasant",
            SeemsLikeAdjective::default(),
            "smells unpleasant",
        );
    }

    #[test]
    fn tastes() {
        assert_suggestion_result(
            "tastes like yucky",
            SeemsLikeAdjective::default(),
            "tastes yucky",
        );
    }
}
