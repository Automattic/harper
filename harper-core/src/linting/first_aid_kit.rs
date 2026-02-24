use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct FirstAidKit {
    expr: Box<dyn Expr>,
}

impl Default for FirstAidKit {
    fn default() -> Self {
        let supply_words = WordSet::new(&["aid", "starter", "travel", "tool"]);
        let pattern = SequenceExpr::with(supply_words)
            .then_whitespace()
            .then_any_capitalization_of("kid");
        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for FirstAidKit {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        let typo_token = tokens.last()?;
        let typo_span = typo_token.span;
        let typo_text = typo_span.get_content(source);
        Some(Lint {
            span: typo_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "kit".chars().collect(),
                typo_text,
            )],
            message: "Did you mean `kit` (a set of items) instead of “kid”?".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Detects when “kid” after “aid”, “starter”, “travel”, or “tool” should be “kit” (a set of supplies)."
    }
}

#[cfg(test)]
mod tests {
    use super::FirstAidKit;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_first_aid_kid() {
        assert_suggestion_result(
            "A first aid kid is a collection of medical supplies.",
            FirstAidKit::default(),
            "A first aid kit is a collection of medical supplies.",
        );
    }

    #[test]
    fn corrects_starter_kid() {
        assert_suggestion_result(
            "Check the starter kid before proceeding.",
            FirstAidKit::default(),
            "Check the starter kit before proceeding.",
        );
    }

    #[test]
    fn corrects_travel_kid() {
        assert_suggestion_result(
            "Pack your travel kid for the trip.",
            FirstAidKit::default(),
            "Pack your travel kit for the trip.",
        );
    }

    #[test]
    fn corrects_tool_kid() {
        assert_suggestion_result(
            "Don't forget the tool kid for assembly.",
            FirstAidKit::default(),
            "Don't forget the tool kit for assembly.",
        );
    }

    #[test]
    fn does_not_flag_kid_in_other_contexts() {
        assert_lint_count(
            "The kid ran through the aid station.",
            FirstAidKit::default(),
            0,
        );
    }
}
