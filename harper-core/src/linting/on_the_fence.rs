use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{AnchorEnd, Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_index_matching},
    },
    patterns::WordSet,
};

pub struct OnTheFence {
    expr: SequenceExpr,
}

impl Default for OnTheFence {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_of([
                Box::new(WordSet::new(&[
                    // "it's" is intentionally omitted
                    "i'm", "we're", "you're", "he's", "she's", "they're",
                ])) as Box<dyn Expr>,
                Box::new(SequenceExpr::aco("i").t_ws().t_set(&["am", "was"])),
                Box::new(
                    SequenceExpr::word_set(&["we", "you", "they"])
                        .t_ws()
                        .t_set(&["are", "were"]),
                ),
                Box::new(
                    // "it" is intentionally omitted
                    SequenceExpr::word_set(&["he", "she", "anybody", "anyone"])
                        .t_ws()
                        .t_set(&["is", "was"]),
                ),
            ])
            .then_optional(SequenceExpr::whitespace().t_set(&["also", "still"]))
            .t_ws()
            .then_word_seq(&["on", "a", "fence"])
            .then_any_of([
                Box::new(AnchorEnd) as Box<dyn Expr>,
                Box::new(SequenceExpr::whitespace().then_any_of([
                    Box::new(SequenceExpr::default().then_preposition()) as Box<dyn Expr>,
                    Box::new(AnchorEnd),
                ])),
            ]),
        }
    }
}

impl ExprLinter for OnTheFence {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let fence_index =
            find_the_only_token_index_matching(toks, src, |t, s| t.get_ch(s).eq_str("fence"))?;
        let span = toks.get(fence_index - 2)?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "the",
                span.get_content(src),
            )],
            message: "If this is the idiom meaning `undecided`, it should be `on the fence`."
                .to_owned(),
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
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::OnTheFence;

    // Contrived minimal tests

    #[test]
    fn on_the_fence_end_no_punc() {
        assert_suggestion_result("I'm on a fence", OnTheFence::default(), "I'm on the fence");
    }

    #[test]
    fn on_the_fence_end_period() {
        assert_suggestion_result(
            "You're on a fence.",
            OnTheFence::default(),
            "You're on the fence.",
        );
    }

    #[test]
    fn on_the_fence_end_space_period() {
        assert_suggestion_result(
            "He is on a fence .",
            OnTheFence::default(),
            "He is on the fence .",
        );
    }

    // Real-world tests

    #[test]
    fn fix_anyone_is_on_a_fence() {
        assert_suggestion_result(
            "If anyone is still on a fence, the new Joyride update changes a lot.",
            OnTheFence::default(),
            "If anyone is still on the fence, the new Joyride update changes a lot.",
        );
    }

    #[test]
    fn fix_i_am_about_this() {
        assert_suggestion_result(
            "I am on a fence about this.",
            OnTheFence::default(),
            "I am on the fence about this.",
        );
    }

    #[test]
    fn fix_i_am_also() {
        assert_suggestion_result(
            "I am also on a fence, especially b/c the morning check-in might end up being a duplicate of previous EOD meeting",
            OnTheFence::default(),
            "I am also on the fence, especially b/c the morning check-in might end up being a duplicate of previous EOD meeting",
        );
    }

    #[test]
    fn fix_im_of() {
        assert_suggestion_result(
            "I'm on a fence of scripting it myself for good or improving the initial experience for everyone :)",
            OnTheFence::default(),
            "I'm on the fence of scripting it myself for good or improving the initial experience for everyone :)",
        );
    }

    #[test]
    fn fix_im_still_with() {
        assert_suggestion_result(
            "I'm still on a fence with Rust at this point.",
            OnTheFence::default(),
            "I'm still on the fence with Rust at this point.",
        );
    }

    #[test]
    fn fix_you_are_of() {
        assert_suggestion_result(
            "They have a one month contract so if you are on a fence of trying them out, that's a great starter option. ",
            OnTheFence::default(),
            "They have a one month contract so if you are on the fence of trying them out, that's a great starter option. ",
        );
    }

    #[test]
    fn dont_flag_it() {
        assert_no_lints(
            "I would like to remove it but it is on a fence between the neighbors yard and mine.",
            OnTheFence::default(),
        );
    }

    #[test]
    fn dont_flag_posted() {
        assert_no_lints(
            "A private property sign is posted on a fence.",
            OnTheFence::default(),
        );
    }

    #[test]
    fn dont_flag_scrolling() {
        assert_no_lints(
            "My only issue is scrolling on a fence is a bit slow and laggy.",
            OnTheFence::default(),
        );
    }

    #[test]
    fn dont_flag_verse() {
        assert_no_lints(
            "My daddy is a dollar / I wrote it on a fence / My daddy is a dollar / not worth a hundred cents.",
            OnTheFence::default(),
        );
    }
}
