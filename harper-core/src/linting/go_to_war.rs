use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct GoToWar {
    expr: Box<dyn Expr>,
}

impl Default for GoToWar {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["go", "goes", "going", "gone", "went"])
                    .t_ws()
                    .then_preposition()
                    .t_ws()
                    .then_word_set(&["war"]),
            ),
        }
    }
}

impl ExprLinter for GoToWar {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Replaces `go at war` with `go to war`."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let prep_idx = 2;
        let prep_tok = &toks[prep_idx];
        let prep_span = prep_tok.span;
        let prep_chars = prep_span.get_content(src);

        if prep_chars.eq_ignore_ascii_case_chars(&['t', 'o']) {
            return None;
        }

        Some(Lint {
            span: prep_span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str("to", prep_chars)],
            message: "Use `to` instead of `at`.".to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::GoToWar;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn go_at() {
        assert_suggestion_result(
            "specialization makes you vulnerable if you go at war with your trading partners",
            GoToWar::default(),
            "specialization makes you vulnerable if you go to war with your trading partners",
        );
    }

    #[test]
    fn go_in() {
        assert_suggestion_result(
            "for whatever reason, it would go in war with another town",
            GoToWar::default(),
            "for whatever reason, it would go to war with another town",
        );
    }

    #[test]
    fn go_on() {
        assert_suggestion_result(
            "How much time do we have before Youtube starts to go on war with Revanced?",
            GoToWar::default(),
            "How much time do we have before Youtube starts to go to war with Revanced?",
        );
    }

    #[test]
    fn goes_on() {
        assert_suggestion_result(
            "It would be the same case if USA goes on war with Canada and Mexico.",
            GoToWar::default(),
            "It would be the same case if USA goes to war with Canada and Mexico.",
        );
    }

    #[test]
    fn going_at() {
        assert_suggestion_result(
            "So instead of going at war with technology, let's be friends and work better.",
            GoToWar::default(),
            "So instead of going to war with technology, let's be friends and work better.",
        );
    }

    #[test]
    fn going_on() {
        assert_suggestion_result(
            "How consequences of India going on war with Pakistan after the recent Uri Terror attack?",
            GoToWar::default(),
            "How consequences of India going to war with Pakistan after the recent Uri Terror attack?",
        );
    }

    #[test]
    fn went_at() {
        assert_suggestion_result(
            "the magic energy released since the colleges went at war with each others",
            GoToWar::default(),
            "the magic energy released since the colleges went to war with each others",
        );
    }

    #[test]
    fn went_in() {
        assert_suggestion_result(
            "even America wanted to expand its territories and they went in War with Mexico",
            GoToWar::default(),
            "even America wanted to expand its territories and they went to War with Mexico",
        );
    }

    #[test]
    fn went_on() {
        assert_suggestion_result(
            "I used to skip clubman and make as many vills as i can and then went on war with ai",
            GoToWar::default(),
            "I used to skip clubman and make as many vills as i can and then went to war with ai",
        );
    }
}
