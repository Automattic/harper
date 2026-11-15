use crate::{
    Lint, Token, TokenStringExt,
    expr::{AnchorStart, Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct ExpandIme {
    expr: SequenceExpr,
}

impl Default for ExpandIme {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::with(AnchorStart).t_aco("ime"),
        }
    }
}

impl ExprLinter for ExpandIme {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));
        let span = matched_tokens.span()?;
        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            // Don't use `::replace_with_match_case` since it sets case by matching letter index.
            // Since it's canonically a sentence-initial adverb, use title case.
            // If this turns out to be a problem after a comma, we can revisit.
            suggestions: vec![Suggestion::ReplaceWith(
                "In my experience".chars().collect(),
            )],
            message: "Use `in my experience` instead of `ime`, if that's what you mean.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Expands the abbreviation `ime` to the full phrase `in my experience` for clarity."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ExpandIme;

    // Contrived basic functionality tets

    #[test]
    fn after_comma_space() {
        assert_suggestion_result(
            "foo bar, ime baz",
            ExpandIme::default(),
            "foo bar, In my experience baz",
        );
    }

    #[test]
    fn after_comma_no_space() {
        assert_suggestion_result(
            "foo bar,ime baz",
            ExpandIme::default(),
            "foo bar,In my experience baz",
        );
    }

    #[test]
    fn after_full_stop() {
        assert_suggestion_result(
            "foo bar. ime baz",
            ExpandIme::default(),
            "foo bar. In my experience baz",
        );
    }

    #[test]
    fn after_full_stop_no_space() {
        assert_suggestion_result(
            "foo bar.ime baz",
            ExpandIme::default(),
            "foo bar.In my experience baz",
        );
    }

    // Real-world tests

    #[test]
    fn fix_at_start_of_sentence_with_comma() {
        assert_suggestion_result(
            "IME, at least a weak of full-time dedication is needed, with a suitable editor and config, to start grasping this.",
            ExpandIme::default(),
            "In my experience, at least a weak of full-time dedication is needed, with a suitable editor and config, to start grasping this.",
        );
    }

    #[test]
    fn fix_at_start_of_sentence_without_comma() {
        assert_suggestion_result(
            "IME these types of tests are slow, brittle and don't give you much value.",
            ExpandIme::default(),
            "In my experience these types of tests are slow, brittle and don't give you much value.",
        );
    }

    #[test]
    fn dont_flag_mid_chunk() {
        assert_no_lints(
            "I have an IME that allows me to enter characters by their unicode code point.",
            ExpandIme::default(),
        );
    }
}
