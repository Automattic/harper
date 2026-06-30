use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_token},
    },
};

pub struct CallItQuits {
    expr: SequenceExpr,
}

impl Default for CallItQuits {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["call", "calls", "called", "calling"])
                .t_ws()
                .t_aco("it")
                .t_ws()
                // "Quit" and "QUIT" would result in many false positives.
                .then_exact_word("quit"),
        }
    }
}

impl ExprLinter for CallItQuits {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        _source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // Watch out for calling a funciton `quit()`
        if followed_by_token(context, |t| t.kind.is_open_round()) {
            return None;
        }

        Some(Lint {
            span: matched_tokens.last()?.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::ReplaceWith("quits".chars().collect())],
            message: "This idiom uses the plural 'quits' to mean 'gives up' or 'stops'.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects wrong variants of the idiom 'call it quits'."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::CallItQuits;

    #[test]
    fn fix_cant_now() {
        assert_suggestion_result(
            "You can't call it quit now, you need to see it through !!",
            CallItQuits::default(),
            "You can't call it quits now, you need to see it through !!",
        );
    }

    #[test]
    fn fix_for_the_evening() {
        assert_suggestion_result(
            "do some maintenance and logistics before you call it quit for the evening",
            CallItQuits::default(),
            "do some maintenance and logistics before you call it quits for the evening",
        );
    }

    #[test]
    fn fix_before_calling() {
        assert_suggestion_result(
            "you may like to put a large number for TIMEOUT, so that before calling it quit, xstress will wait to see",
            CallItQuits::default(),
            "you may like to put a large number for TIMEOUT, so that before calling it quits, xstress will wait to see",
        );
    }

    #[test]
    fn fix_might_need_to() {
        assert_suggestion_result(
            "But I feel like I might need to call it quit and calibrate/find parameters myself and ignore the data provided/used inside OpenVR.",
            CallItQuits::default(),
            "But I feel like I might need to call it quits and calibrate/find parameters myself and ignore the data provided/used inside OpenVR.",
        );
    }

    #[test]
    fn fix_finally() {
        assert_suggestion_result(
            "I was about to call it quit but I tried a second time this way and it finally worked",
            CallItQuits::default(),
            "I was about to call it quits but I tried a second time this way and it finally worked",
        );
    }

    #[test]
    fn fix_ctf() {
        assert_suggestion_result(
            "everyone got tired, we called it quit and the CTF ended",
            CallItQuits::default(),
            "everyone got tired, we called it quits and the CTF ended",
        );
    }

    #[test]
    fn avoid_some_call_it() {
        assert_no_lints(
            "Some call it Quit, some Exit and some Close, There's no issue to fix.",
            CallItQuits::default(),
        );
    }

    #[test]
    fn avoid_lets_call_it() {
        assert_no_lints(
            "and i need a new UiMode variant let's call it Quit.",
            CallItQuits::default(),
        );
    }

    #[test]
    fn dont_flg_wither_intervenin_punctuation() {
        assert_no_lints(
            "I try it, but I can't find function of dispose called. it quit immediately.",
            CallItQuits::default(),
        );
    }

    #[test]
    fn avoid_flagging_all_caps_possible_false_positives() {
        assert_no_lints("Just call it QUIT and you're done.", CallItQuits::default());
    }

    #[test]
    fn avoid_flagging_function_names() {
        assert_no_lints(
            "call it soloSplit() , call it quit() or whatever else would be nice",
            CallItQuits::default(),
        );
    }

    #[test]
    fn avoid_flagging_stop_button() {
        assert_no_lints(
            "The Stop button (or maybe call it Quit) should stop just like the Pause button does",
            CallItQuits::default(),
        );
    }
}
