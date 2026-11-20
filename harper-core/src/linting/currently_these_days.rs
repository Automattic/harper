use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct CurrentlyTheseDays {
    expr: FirstMatchOf,
}

impl Default for CurrentlyTheseDays {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::aco("currently")
                        .t_ws()
                        .then_zero_or_more(
                            SequenceExpr::with(|t: &Token, s: &[char]| {
                                t.kind.is_word() && !t.get_ch(s).eq_str("these")
                            })
                            .t_ws(),
                        )
                        .then_word_seq(&["these", "days"]),
                ),
                Box::new(
                    SequenceExpr::word_seq(&["these", "days"])
                        .t_ws()
                        .then_zero_or_more(
                            SequenceExpr::with(|t: &Token, s: &[char]| {
                                t.kind.is_word() && !t.get_ch(s).eq_str("currently")
                            })
                            .t_ws(),
                        )
                        .t_aco("currently"),
                ),
                // Box::new(SequenceExpr::default().then_zero_or_more_spaced(SequenceExpr::any_word())),
            ]),
        }
    }
}

impl ExprLinter for CurrentlyTheseDays {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if matched_tokens.len() == 1
            && matched_tokens[0]
                .get_ch(source)
                .eq_any_ignore_ascii_case_str(&["foo", "bar", "baz", "qux"])
        {
            return None;
        }
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));

        let span = matched_tokens.span()?;
        let lint_kind = LintKind::Miscellaneous;
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "foo",
            span.get_content(source),
        )];
        let message = "Fix this erorr".to_owned();
        Some(Lint {
            span,
            lint_kind,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter skeleton for contributors to copy into `harper_core/src/linting/` and rename."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_suggestion_result};

    use super::CurrentlyTheseDays;

    #[test]
    fn holotype() {
        assert_good_and_bad_suggestions(
            "Now this app is called Floorplan, and it's an open source project and is currently getting a lot of traction these days.",
            CurrentlyTheseDays::default(),
            &[
                "Now this app is called Floorplan, and it's an open source project and is currently getting a lot of traction.",
                "Now this app is called Floorplan, and it's an open source project and is getting a lot of traction these days.",
            ],
            &[],
        );
    }

    #[test]
    fn c_td() {
        assert_suggestion_result(
            "currently I like redundancy these days",
            CurrentlyTheseDays::default(),
            "foo",
        );
    }

    #[test]
    fn td_c() {
        assert_suggestion_result(
            "these days I like redundancy currently",
            CurrentlyTheseDays::default(),
            "foo",
        );
    }

    #[test]
    fn just_a_single_space() {
        assert_suggestion_result(" ", CurrentlyTheseDays::default(), "bar");
    }

    #[test]
    fn a_space_between_words() {
        assert_suggestion_result("foo bar", CurrentlyTheseDays::default(), "baz");
    }

    #[test]
    fn three_words() {
        assert_suggestion_result("foo bar baz", CurrentlyTheseDays::default(), "qux");
    }
}
