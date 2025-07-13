use crate::{
    Token,
    expr::SequenceExpr,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct HopeYoure {
    expr: Box<dyn crate::expr::Expr>,
}

impl Default for HopeYoure {
    fn default() -> Self {
        let loc = WordSet::new(&["here", "there"]);

        let prep = SequenceExpr::default().t_ws().then_preposition();

        let expr = SequenceExpr::aco("hope")
            .t_ws()
            .t_aco("your")
            .t_ws()
            .then_adjective()
            .then_optional(prep)
            .t_ws()
            .then(loc);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for HopeYoure {
    fn expr(&self) -> &dyn crate::expr::Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let your_tok = toks.get(2)?;
        let span = your_tok.span;
        let original = span.get_content(src);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "you're".chars().collect(),
                original,
            )],
            message: "Prefer `you're`—the contraction of “you are”—when expressing a hope about someone’s condition."
                .into(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Detects the misuse of possessive **your** after “hope” and an adjective \
         (e.g., “I hope your well”) and advises using **you’re** to supply the \
         missing verb “are.”"
    }
}

#[cfg(test)]
mod tests {
    use super::HopeYoure;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_excited_here() {
        assert_suggestion_result(
            "I hope your excited here.",
            HopeYoure::default(),
            "I hope you're excited here.",
        );
    }

    #[test]
    fn corrects_safe_there() {
        assert_suggestion_result(
            "I hope your safe there.",
            HopeYoure::default(),
            "I hope you're safe there.",
        );
    }

    #[test]
    fn corrects_safe_over_there() {
        assert_suggestion_result(
            "I hope your safe over there.",
            HopeYoure::default(),
            "I hope you're safe over there.",
        );
    }

    #[test]
    fn corrects_fine_here() {
        assert_suggestion_result(
            "I hope your fine here.",
            HopeYoure::default(),
            "I hope you're fine here.",
        );
    }

    #[test]
    fn corrects_happy_there() {
        assert_suggestion_result(
            "I hope your happy there.",
            HopeYoure::default(),
            "I hope you're happy there.",
        );
    }

    #[test]
    fn corrects_healthy_out_here() {
        assert_suggestion_result(
            "I hope your healthy out here.",
            HopeYoure::default(),
            "I hope you're healthy out here.",
        );
    }

    #[test]
    fn corrects_strong_there() {
        assert_suggestion_result(
            "We hope your strong there.",
            HopeYoure::default(),
            "We hope you're strong there.",
        );
    }

    #[test]
    fn corrects_sorry_here() {
        assert_suggestion_result(
            "Hope your sorry here.",
            HopeYoure::default(),
            "Hope you're sorry here.",
        );
    }

    #[test]
    fn no_lint_with_contraction() {
        assert_lint_count("I hope you're excited here.", HopeYoure::default(), 0);
    }

    #[test]
    fn no_lint_without_adjective() {
        assert_lint_count("I hope your trip went well.", HopeYoure::default(), 0);
    }

    #[test]
    fn no_lint_with_following_clause() {
        assert_lint_count(
            "I hope your friends are well there.",
            HopeYoure::default(),
            0,
        );
    }

    #[test]
    fn no_lint_when_possessive_context() {
        assert_lint_count(
            "I hope your advice on punctuation is helpful.",
            HopeYoure::default(),
            0,
        );
    }
}
