use crate::Token;
use crate::expr::{Expr, SequenceExpr};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ApartFrom {
    expr: Box<dyn Expr>,
}

impl Default for ApartFrom {
    fn default() -> Self {
        let expr = SequenceExpr::any_capitalization_of("apart")
            .t_ws()
            .then_any_capitalization_of("form");

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ApartFrom {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.last()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "from",
                span.get_content(source),
            )],
            message: "Use `from` to spell `apart from`.".to_owned(),
            priority: 50,
        })
    }

    fn description(&self) -> &'static str {
        "Flags the misspelling `apart form` and suggests `apart from`."
    }
}

#[cfg(test)]
mod tests {
    use super::ApartFrom;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_basic_typo() {
        assert_suggestion_result(
            "Christianity was set apart form other religions.",
            ApartFrom::default(),
            "Christianity was set apart from other religions.",
        );
    }

    #[test]
    fn corrects_title_case() {
        assert_suggestion_result(
            "Apart Form these files, everything uploaded fine.",
            ApartFrom::default(),
            "Apart From these files, everything uploaded fine.",
        );
    }

    #[test]
    fn corrects_all_caps() {
        assert_suggestion_result(
            "APART FORM THE REST OF THE FIELD.",
            ApartFrom::default(),
            "APART FROM THE REST OF THE FIELD.",
        );
    }

    #[test]
    fn corrects_with_comma() {
        assert_suggestion_result(
            "It was apart form, not apart from, the original plan.",
            ApartFrom::default(),
            "It was apart from, not apart from, the original plan.",
        );
    }

    #[test]
    fn corrects_with_newline() {
        assert_suggestion_result(
            "They stood apart\nform everyone else at the rally.",
            ApartFrom::default(),
            "They stood apart\nfrom everyone else at the rally.",
        );
    }

    #[test]
    fn corrects_extra_spacing() {
        assert_suggestion_result(
            "We keep the archive apart   form public assets.",
            ApartFrom::default(),
            "We keep the archive apart   from public assets.",
        );
    }

    #[test]
    fn allows_correct_phrase() {
        assert_lint_count(
            "Lebanon's freedoms set it apart from other Arab states.",
            ApartFrom::default(),
            0,
        );
    }

    #[test]
    fn ignores_hyphenated() {
        assert_lint_count(
            "Their apart-form design wasn’t what we needed.",
            ApartFrom::default(),
            0,
        );
    }

    #[test]
    fn ignores_split_by_comma() {
        assert_lint_count(
            "They stood apart, form lines when asked.",
            ApartFrom::default(),
            0,
        );
    }

    #[test]
    fn ignores_unrelated_form_usage() {
        assert_lint_count(
            "The form was kept apart to dry after printing.",
            ApartFrom::default(),
            0,
        );
    }
}
