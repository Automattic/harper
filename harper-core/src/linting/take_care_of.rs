use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct TakeCareOf {
    expr: SequenceExpr,
}

impl Default for TakeCareOf {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["take", "taken", "takes", "taking", "took"])
                .t_ws()
                .then_word_seq(&["care", "about"]),
        }
    }
}

impl ExprLinter for TakeCareOf {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.last()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "of",
                span.get_content(src),
            )],
            message: "Are you confusing `care about` with `take care of`?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `take care about` to `take care of`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::TakeCareOf;

    #[test]
    fn fix_take() {
        assert_suggestion_result(
            "OSGi: Take care about dynamically changed routes (and other initial data)",
            TakeCareOf::default(),
            "OSGi: Take care of dynamically changed routes (and other initial data)",
        );
    }

    #[test]
    fn fix_taken() {
        assert_suggestion_result(
            "But this would mean that the creator of the logfile has taken care about making the fraction 6 characters long",
            TakeCareOf::default(),
            "But this would mean that the creator of the logfile has taken care of making the fraction 6 characters long",
        );
    }

    #[test]
    fn fix_takes() {
        assert_suggestion_result(
            "I like that Dart actively takes care about this and provides this consistent package",
            TakeCareOf::default(),
            "I like that Dart actively takes care of this and provides this consistent package",
        );
    }

    #[test]
    fn fix_taking() {
        assert_suggestion_result(
            "second one is taking care about analysis of output",
            TakeCareOf::default(),
            "second one is taking care of analysis of output",
        );
    }

    #[test]
    fn fix_took() {
        assert_suggestion_result(
            "then this * function took care about the ndisc option",
            TakeCareOf::default(),
            "then this * function took care of the ndisc option",
        );
    }
}
