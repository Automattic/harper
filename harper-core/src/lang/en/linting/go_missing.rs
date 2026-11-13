use crate::{
    CharStringExt, Lint, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct GoMissing {
    expr: All,
}

impl Default for GoMissing {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["become", "became", "becomes", "becoming"])
                .t_ws()
                .t_aco("missing")
                .but_not(SequenceExpr::anything().t_any().t_any().t_ws().then(
                    |t: &Token, s: &[char]| {
                        t.kind.is_plural_noun()
                            || t.get_ch(s)
                                .eq_any_ignore_ascii_case_str(&["child", "person"])
                    },
                )),
        }
    }
}

impl ExprLinter for GoMissing {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.first()?.span;
        let ch = span.get_content(src);

        let corrections: &[&str] = if ch.eq_str("became") {
            &["went"]
        } else if ch.eq_str("become") {
            &["go", "gone"]
        } else if ch.eq_str("becomes") {
            &["goes"]
        } else if ch.eq_str("becoming") {
            &["going"]
        } else {
            return None;
        };

        let suggestions = corrections
            .iter()
            .map(|c| Suggestion::replace_with_match_case_str(c, ch))
            .collect::<Vec<_>>();

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "The idiomatic expression is `go missing`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `become missing` to `go missing`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::GoMissing;

    #[test]
    fn fix_become_missing() {
        assert_suggestion_result(
            "the step where chroma/saturation is checked and hue might become missing",
            GoMissing::default(),
            "the step where chroma/saturation is checked and hue might go missing",
        );
    }

    #[test]
    fn fix_become_missing_title_case() {
        assert_suggestion_result(
            "How Do Quilts Become Missing?",
            GoMissing::default(),
            "How Do Quilts Go Missing?",
        )
    }

    #[test]
    fn fix_becomes_missing() {
        assert_suggestion_result(
            "One of them becomes missing, but since the new one was already in the library",
            GoMissing::default(),
            "One of them goes missing, but since the new one was already in the library",
        );
    }

    #[test]
    fn fix_becoming_missing() {
        assert_suggestion_result(
            "... could have confusing semantics of flows becoming missing arbitrarily",
            GoMissing::default(),
            "... could have confusing semantics of flows going missing arbitrarily",
        )
    }

    #[test]
    fn fix_became_missing() {
        assert_suggestion_result(
            "the 'data-target' attribute from the link became missing",
            GoMissing::default(),
            "the 'data-target' attribute from the link went missing",
        )
    }

    #[test]
    fn fix_has_become_missing() {
        assert_suggestion_result(
            "Consider it correct that a reference to an asset that has become missing once is not restored.",
            GoMissing::default(),
            "Consider it correct that a reference to an asset that has become missing once is not restored.",
        )
    }

    #[test]
    fn fix_have_become_missing() {
        assert_suggestion_result(
            "... and values that may have become missing ...",
            GoMissing::default(),
            "... and values that may have become missing ...",
        )
    }

    #[test]
    fn dont_flag_missing_persons() {
        assert_no_lints(
            "Prevent Them Becoming Missing Persons",
            GoMissing::default(),
        )
    }

    #[test]
    fn dont_flag_missing_children() {
        assert_no_lints(
            "and unaccompanied children who become missing children",
            GoMissing::default(),
        )
    }

    #[test]
    fn dont_flag_missing_child() {
        assert_no_lints(
            "She became Missing Child Number 1201258.",
            GoMissing::default(),
        )
    }
}
