use crate::expr::{Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};
use crate::{CharStringExt, Token};

/// Corrects `way` when it is likely a typo for the verb `wait`.
pub struct WayWait {
    expr: SequenceExpr,
}

impl Default for WayWait {
    fn default() -> Self {
        let negative_auxiliary = SequenceExpr::any_of(vec![
            Box::new(SequenceExpr::word_set(&[
                "can't",
                "cant",
                "cannot",
                "couldn't",
                "couldnt",
                "doesn't",
                "doesnt",
                "don't",
                "dont",
                "mightn't",
                "mightnt",
                "mustn't",
                "mustnt",
                "shan't",
                "shant",
                "shouldn't",
                "shouldnt",
                "won't",
                "wont",
            ])),
            Box::new(SequenceExpr::fixed_phrase("can not")),
        ]);

        Self {
            expr: SequenceExpr::with(negative_auxiliary)
                .t_ws()
                .t_aco("way")
                .t_ws()
                .t_set(&["for", "to", "until"]),
        }
    }
}

impl ExprLinter for WayWait {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let way_span = toks
            .iter()
            .find(|tok| tok.get_ch(src).eq_any_ignore_ascii_case_str(&["way"]))?
            .span;

        Some(Lint {
            span: way_span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case(
                "wait".chars().collect(),
                way_span.get_content(src),
            )],
            message: "‘Way’ might be a typo here. Did you mean ‘wait’?".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects `way` when it appears to be a typo for `wait`."
    }
}

#[cfg(test)]
mod tests {
    use super::WayWait;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn fix_cant_way_to() {
        assert_suggestion_result(
            "Wow, just seeing this again, incredible, can't way to try it.",
            WayWait::default(),
            "Wow, just seeing this again, incredible, can't wait to try it.",
        );
    }

    #[test]
    fn fix_cannot_way_for() {
        assert_suggestion_result(
            "If it's a real bother and you cannot way for 2.10.16, go back to 2.10.12.",
            WayWait::default(),
            "If it's a real bother and you cannot wait for 2.10.16, go back to 2.10.12.",
        );
    }

    #[test]
    fn fix_doesnt_way_until() {
        assert_suggestion_result(
            "It doesn't way until completion though.",
            WayWait::default(),
            "It doesn't wait until completion though.",
        );
    }

    #[test]
    fn fix_can_not_way_for() {
        assert_suggestion_result(
            "I can not way for something better to be released.",
            WayWait::default(),
            "I can not wait for something better to be released.",
        );
    }

    #[test]
    fn fix_negative_modals() {
        assert_suggestion_result(
            "You shouldn't way to tell them.",
            WayWait::default(),
            "You shouldn't wait to tell them.",
        );
        assert_suggestion_result(
            "We mustn't way until completion.",
            WayWait::default(),
            "We mustn't wait until completion.",
        );
        assert_suggestion_result(
            "I shan't way for another release.",
            WayWait::default(),
            "I shan't wait for another release.",
        );
    }

    #[test]
    fn dont_flag_unclear_way_uses() {
        assert_no_lints(
            "The RFC doesn't way what resolved value means.",
            WayWait::default(),
        );
    }

    #[test]
    fn dont_flag_correct_wait() {
        assert_no_lints("I can't wait to try it.", WayWait::default());
    }
}
