use super::{token_is_likely_their_possession, token_is_typographic_theyre};
use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    expr::SequenceExpr,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct TypographicTheyreToTheir {
    expr: Box<dyn crate::expr::Expr>,
}

impl Default for TypographicTheyreToTheir {
    fn default() -> Self {
        let expr = SequenceExpr::with(token_is_typographic_theyre as fn(&Token, &[char]) -> bool)
            .t_ws()
            .then(token_is_likely_their_possession as fn(&Token, &[char]) -> bool);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for TypographicTheyreToTheir {
    type Unit = Chunk;

    fn expr(&self) -> &dyn crate::expr::Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offender = matched_tokens.first()?;
        let template = offender.span.get_content(source);

        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case_str("their", template)],
            message: "Did you mean `their`?".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects smart-apostrophe `they’re` to `their` in possessive noun contexts."
    }
}

#[cfg(test)]
mod tests {
    use super::TypographicTheyreToTheir;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_smart_apostrophe_possessive() {
        assert_suggestion_result(
            "I think they’re house is the blue one.",
            TypographicTheyreToTheir::default(),
            "I think their house is the blue one.",
        );
    }

    #[test]
    fn ignores_ascii_form_for_existing_rule() {
        assert_lint_count(
            "I think they're house is the blue one.",
            TypographicTheyreToTheir::default(),
            0,
        );
    }

    #[test]
    fn ignores_non_possessive_usage() {
        assert_lint_count(
            "I think they’re coming tonight.",
            TypographicTheyreToTheir::default(),
            0,
        );
    }

    #[test]
    fn ignores_contraction_examples_from_books() {
        assert_lint_count("No, they’re not.", TypographicTheyreToTheir::default(), 0);
        assert_lint_count(
            "I don’t know what they’re like.",
            TypographicTheyreToTheir::default(),
            0,
        );
        assert_lint_count(
            "They’re all over crumbs.",
            TypographicTheyreToTheir::default(),
            0,
        );
        assert_lint_count(
            "They’re done with blacking, I believe.",
            TypographicTheyreToTheir::default(),
            0,
        );
    }
}
