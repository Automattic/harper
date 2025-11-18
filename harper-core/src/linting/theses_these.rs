use harper_brill::UPOS;
use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::patterns::UPOSSet;
use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ThesesThese {
    expr: Box<dyn Expr>,
}

impl Default for ThesesThese {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .t_aco("theses")
            .t_ws()
            .then(UPOSSet::new(&[UPOS::NOUN, UPOS::PROPN]));

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ThesesThese {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let theses_token = matched_tokens.first()?;

        let content = theses_token.span.get_content(source);

        let suggestions = vec![Suggestion::replace_with_match_case_str("these", content)];

        Some(Lint {
            span: theses_token.span,
            lint_kind: LintKind::Spelling,
            suggestions,
            message: "Did you mean “these”?".to_string(),
            priority: 1,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects the common misspelling of `these` as `theses`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    use super::ThesesThese;

    #[test]
    fn corrects_theses_scenes() {
        assert_suggestion_result(
            "Are theses scenes from a novel?",
            ThesesThese::default(),
            "Are these scenes from a novel?",
        );
    }

    #[test]
    fn corrects_theses_days() {
        assert_suggestion_result(
            "That's why the two countries look as they do theses days.",
            ThesesThese::default(),
            "That's why the two countries look as they do these days.",
        );
    }

    #[test]
    fn allows_correct_theses() {
        assert_no_lints(
            "There are universities that are dedicated just to this field, thousands of people doing theses on Picasso, for example.",
            ThesesThese::default(),
        );
    }

    #[test]
    fn allows_theses_followed_by_verb() {
        assert_no_lints(
            "Theses are the times that try men's souls.",
            ThesesThese::default(),
        );
    }

    #[test]
    fn works_with_capitalization() {
        assert_suggestion_result(
            "THESES BOOKS ARE GREAT.",
            ThesesThese::default(),
            "THESE BOOKS ARE GREAT.",
        );
    }

    #[test]
    fn works_with_mixed_capitalization() {
        assert_suggestion_result(
            "Theses Books Are My Favorite.",
            ThesesThese::default(),
            "These Books Are My Favorite.",
        );
    }

    #[test]
    fn simple_case() {
        assert_suggestion_result(
            "I like theses apples.",
            ThesesThese::default(),
            "I like these apples.",
        );
    }

    #[test]
    fn with_punctuation() {
        assert_no_lints(
            "Are theses, books good?",
            ThesesThese::default(),
        );
    }

    #[test]
    fn in_the_middle_of_sentence() {
        assert_suggestion_result(
            "I saw theses movies yesterday.",
            ThesesThese::default(),
            "I saw these movies yesterday.",
        );
    }

    #[test]
    fn another_example() {
        assert_lint_count("I have theses books.", ThesesThese::default(), 1);
    }

    #[test]
    fn allows_band_name() {
        assert_no_lints("Theses are a great band.", ThesesThese::default());
    }

    #[test]
    fn does_not_correct_valid_theses() {
        assert_no_lints("She wrote multiple theses on the topic.", ThesesThese::default());
    }
}
