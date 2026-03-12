use crate::{
    CharStringExt, Span, Token, TokenKind,
    expr::{Expr, SequenceExpr},
    linting::{Chunk, ExprLinter, Lint, LintKind, Suggestion},
};

pub struct RedundantProgressiveComparative {
    expr: Box<dyn Expr>,
}

impl Default for RedundantProgressiveComparative {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["increasingly", "progressively", "steadily"])
                    .t_ws()
                    .then_word_set(&["more", "less"])
                    .t_ws()
                    .then_kind_either(TokenKind::is_adjective, TokenKind::is_adverb),
            ),
        }
    }
}

impl ExprLinter for RedundantProgressiveComparative {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], src: &[char]) -> Option<Lint> {
        let first = matched_tokens.first()?;
        let second = matched_tokens.get(2)?;

        if !first.kind.is_degree_adverb() {
            return None;
        }

        let replacement = if second
            .span
            .get_content(src)
            .eq_ignore_ascii_case_str("more")
        {
            "more and more"
        } else if second
            .span
            .get_content(src)
            .eq_ignore_ascii_case_str("less")
        {
            "less and less"
        } else {
            return None;
        };

        let span = Span::new(first.span.start, second.span.end);

        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                replacement,
                span.get_content(src),
            )],
            message: "This phrasing is redundant; use a direct comparative like `more and more`."
                .to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects redundant degree-adverb comparatives like `increasingly more` and `increasingly less`."
    }
}

#[cfg(test)]
mod tests {
    use super::RedundantProgressiveComparative;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn fixes_increasingly_more() {
        assert_suggestion_result(
            "The issue is increasingly more prevalent in distributed systems.",
            RedundantProgressiveComparative::default(),
            "The issue is more and more prevalent in distributed systems.",
        );
    }

    #[test]
    fn fixes_progressively_more() {
        assert_suggestion_result(
            "These failures are progressively more severe in production.",
            RedundantProgressiveComparative::default(),
            "These failures are more and more severe in production.",
        );
    }

    #[test]
    fn fixes_steadily_more() {
        assert_suggestion_result(
            "The interface is steadily more accessible each release.",
            RedundantProgressiveComparative::default(),
            "The interface is more and more accessible each release.",
        );
    }

    #[test]
    fn fixes_increasingly_less() {
        assert_suggestion_result(
            "The outages are increasingly less frequent after the migration.",
            RedundantProgressiveComparative::default(),
            "The outages are less and less frequent after the migration.",
        );
    }

    #[test]
    fn fixes_progressively_less() {
        assert_suggestion_result(
            "The warnings became progressively less noticeable over time.",
            RedundantProgressiveComparative::default(),
            "The warnings became less and less noticeable over time.",
        );
    }

    #[test]
    fn fixes_steadily_less() {
        assert_suggestion_result(
            "The logs are steadily less noisy in recent builds.",
            RedundantProgressiveComparative::default(),
            "The logs are less and less noisy in recent builds.",
        );
    }

    #[test]
    fn preserves_match_case_title() {
        assert_suggestion_result(
            "The bug is Increasingly More Prevalent in nightly builds.",
            RedundantProgressiveComparative::default(),
            "The bug is More and more Prevalent in nightly builds.",
        );
    }

    #[test]
    fn preserves_match_case_all_caps() {
        assert_suggestion_result(
            "The bug is INCREASINGLY MORE PREVALENT in nightly builds.",
            RedundantProgressiveComparative::default(),
            "The bug is MORE AND MORE PREVALENT in nightly builds.",
        );
    }

    #[test]
    fn preserves_match_case_less_all_caps() {
        assert_suggestion_result(
            "The regressions are INCREASINGLY LESS SEVERE with each patch.",
            RedundantProgressiveComparative::default(),
            "The regressions are LESS AND LESS SEVERE with each patch.",
        );
    }

    #[test]
    fn ignores_more_than() {
        assert_no_lints(
            "The issue is increasingly more than a minor annoyance.",
            RedundantProgressiveComparative::default(),
        );
    }

    #[test]
    fn ignores_less_than() {
        assert_no_lints(
            "The issue is increasingly less than a minor annoyance.",
            RedundantProgressiveComparative::default(),
        );
    }

    #[test]
    fn ignores_noun_after_more() {
        assert_no_lints(
            "The issue is increasingly more people reporting crashes.",
            RedundantProgressiveComparative::default(),
        );
    }

    #[test]
    fn ignores_noun_after_less() {
        assert_no_lints(
            "The issue is increasingly less people reporting crashes.",
            RedundantProgressiveComparative::default(),
        );
    }

    #[test]
    fn ignores_non_degree_adverb_more() {
        assert_no_lints(
            "The issue is much more prevalent in this subsystem.",
            RedundantProgressiveComparative::default(),
        );
    }

    #[test]
    fn ignores_non_degree_adverb_less() {
        assert_no_lints(
            "The issue is significantly less prevalent in this subsystem.",
            RedundantProgressiveComparative::default(),
        );
    }

    #[test]
    fn ignores_without_comparator() {
        assert_no_lints(
            "The issue is increasingly prevalent in this subsystem.",
            RedundantProgressiveComparative::default(),
        );
    }

    #[test]
    fn ignores_other_adverbial_comparatives() {
        assert_no_lints(
            "The issue is much more prevalent in this subsystem.",
            RedundantProgressiveComparative::default(),
        );
        assert_no_lints(
            "The issue is significantly less prevalent in this subsystem.",
            RedundantProgressiveComparative::default(),
        );
        assert_no_lints(
            "The issue is ever more prevalent in this subsystem.",
            RedundantProgressiveComparative::default(),
        );
    }

    #[test]
    fn emits_expected_lint_count() {
        assert_lint_count(
            "The trend is increasingly more likely in larger teams.",
            RedundantProgressiveComparative::default(),
            1,
        );
    }
}
