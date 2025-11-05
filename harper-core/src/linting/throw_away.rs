use crate::{
    Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct ThrowAway {
    expr: Box<dyn Expr>,
}

impl Default for ThrowAway {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .t_aco("through")
            .t_ws()
            .t_aco("away");

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ThrowAway {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let typo = matched_tokens.first()?;
        let original = typo.span.get_content(source);

        Some(Lint {
            span: typo.span,
            lint_kind: LintKind::Typo,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("throw", original),
                Suggestion::replace_with_match_case_str("threw", original),
            ],
            message: "Use `throw away` or `threw away`, depending on the tense you need."
                .to_string(),
            priority: 60,
        })
    }

    fn description(&self) -> &str {
        "Finds the typo `through away` and suggests `throw away` or `threw away` instead."
    }
}

#[cfg(test)]
mod tests {
    use super::ThrowAway;
    use crate::linting::tests::{
        assert_lint_count, assert_no_lints, assert_nth_suggestion_result, assert_suggestion_result,
    };

    #[test]
    fn corrects_simple_case() {
        assert_suggestion_result(
            "We through away the old code.",
            ThrowAway::default(),
            "We throw away the old code.",
        );
    }

    #[test]
    fn offers_past_tense_option() {
        assert_nth_suggestion_result(
            "We through away the old code.",
            ThrowAway::default(),
            "We threw away the old code.",
            1,
        );
    }

    #[test]
    fn corrects_sentence_start_capital() {
        assert_suggestion_result(
            "Through away this document when you're done.",
            ThrowAway::default(),
            "Throw away this document when you're done.",
        );
    }

    #[test]
    fn corrects_all_caps_instance() {
        assert_suggestion_result(
            "Please THROUGH AWAY THE TRASH.",
            ThrowAway::default(),
            "Please THROW AWAY THE TRASH.",
        );
    }

    #[test]
    fn corrects_with_extra_whitespace() {
        assert_suggestion_result(
            "We through  away the leftovers.",
            ThrowAway::default(),
            "We throw  away the leftovers.",
        );
    }

    #[test]
    fn does_not_flag_throw_away() {
        assert_no_lints(
            "They throw away the packaging every time.",
            ThrowAway::default(),
        );
    }

    #[test]
    fn does_not_flag_through_tunnel() {
        assert_no_lints(
            "They walked through the tunnel away from danger.",
            ThrowAway::default(),
        );
    }

    #[test]
    fn flags_multiple_occurrences() {
        assert_lint_count(
            "We through away the forks and through away the spoons.",
            ThrowAway::default(),
            2,
        );
    }

    #[test]
    fn does_not_flag_thorough() {
        assert_no_lints(
            "She gave the room a thorough, away-from-home cleaning.",
            ThrowAway::default(),
        );
    }

    #[test]
    fn corrects_with_contraction() {
        assert_suggestion_result(
            "Don't through away your shot.",
            ThrowAway::default(),
            "Don't throw away your shot.",
        );
    }

    #[test]
    fn does_not_flag_spread_words() {
        assert_no_lints(
            "They pushed through as the crowd moved away.",
            ThrowAway::default(),
        );
    }
}
