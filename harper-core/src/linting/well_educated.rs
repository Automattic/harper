use crate::{
    Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    patterns::{WhitespacePattern, WordSet},
};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct WellEducated {
    expr: Box<dyn Expr>,
}

impl Default for WellEducated {
    fn default() -> Self {
        let combined = WordSet::new(&["good-educated"]);

        let separated = SequenceExpr::default()
            .t_aco("good")
            .then_optional(WhitespacePattern)
            .then_hyphen()
            .then_optional(WhitespacePattern)
            .t_aco("educated");

        let expr = SequenceExpr::any_of(vec![Box::new(combined), Box::new(separated)]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for WellEducated {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let original = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case(
                "well-educated".chars().collect(),
                original,
            )],
            message: "Prefer `well-educated` for this compound.".into(),
            priority: 35,
        })
    }

    fn description(&self) -> &'static str {
        "Replaces `good-educated` with the accepted compound `well-educated`."
    }
}

#[cfg(test)]
mod tests {
    use super::WellEducated;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_simple_sentence() {
        assert_suggestion_result(
            "She is good-educated.",
            WellEducated::default(),
            "She is well-educated.",
        );
    }

    #[test]
    fn corrects_in_clause() {
        assert_suggestion_result(
            "The panel found him good-educated and articulate.",
            WellEducated::default(),
            "The panel found him well-educated and articulate.",
        );
    }

    #[test]
    fn corrects_with_modifier() {
        assert_suggestion_result(
            "They considered her very good-educated for her age.",
            WellEducated::default(),
            "They considered her very well-educated for her age.",
        );
    }

    #[test]
    fn corrects_all_caps() {
        assert_suggestion_result(
            "Their mentors are GOOD-EDUCATED leaders.",
            WellEducated::default(),
            "Their mentors are WELL-EDUCATED leaders.",
        );
    }

    #[test]
    fn corrects_title_case() {
        assert_suggestion_result(
            "The report lauded Good-Educated Candidates.",
            WellEducated::default(),
            "The report lauded Well-Educated Candidates.",
        );
    }

    #[test]
    fn corrects_with_quotes() {
        assert_suggestion_result(
            "He called them \"good-educated\" professionals.",
            WellEducated::default(),
            "He called them \"well-educated\" professionals.",
        );
    }

    #[test]
    fn corrects_split_tokens() {
        assert_suggestion_result(
            "Their children are good - educated despite the odds.",
            WellEducated::default(),
            "Their children are well-educated despite the odds.",
        );
    }

    #[test]
    fn allows_well_educated() {
        assert_lint_count("She is well-educated.", WellEducated::default(), 0);
    }

    #[test]
    fn allows_good_education_phrase() {
        assert_lint_count(
            "They received a good education.",
            WellEducated::default(),
            0,
        );
    }

    #[test]
    fn allows_good_to_be_educated() {
        assert_lint_count(
            "It is good to be educated about local history.",
            WellEducated::default(),
            0,
        );
    }
}
