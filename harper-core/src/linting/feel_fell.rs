use crate::Token;
use crate::char_string::CharStringExt;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::linting::expr_linter::find_the_only_token_matching;
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};

pub struct FeelFell {
    expr: Box<dyn Expr>,
}

impl Default for FeelFell {
    fn default() -> Self {
        let with_word_before = SequenceExpr::word_set(&["didn't", "doesn't"])
            .t_ws()
            .t_aco("fell");

        let with_word_after = SequenceExpr::default()
            .t_aco("fell")
            .t_ws()
            .then_word_set(&[
                "comfortable",
                "free",
                "good",
                "I",
                "I'm",
                "it",
                "it's",
                "like",
                "that",
                "we",
                "you",
            ]);

        Self {
            expr: Box::new(SequenceExpr::any_of(vec![
                Box::new(with_word_before),
                Box::new(with_word_after),
            ])),
        }
    }
}

impl ExprLinter for FeelFell {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let fell_token = find_the_only_token_matching(toks, src, |tok, src| {
            tok.span
                .get_content(src)
                .eq_ignore_ascii_case_chars(&['f', 'e', 'l', 'l'])
        })?;

        Some(Lint {
            span: fell_token.span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "feel",
                fell_token.span.get_content(src),
            )],
            message: "It looks like this is a typo, did you mean `feel`?".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Corrects some expressions using `fell` where `feel` is correct."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::FeelFell;

    #[test]
    fn fix_i_fell_like() {
        assert_suggestion_result(
            "But I fell like i am having a knot in my brain ...",
            FeelFell::default(),
            "But I feel like i am having a knot in my brain ...",
        );
    }

    #[test]
    fn fix_if_you_fell_like_it() {
        assert_suggestion_result(
            "If you fell like it create w2ui-postgres for server side implementation",
            FeelFell::default(),
            "If you feel like it create w2ui-postgres for server side implementation",
        );
    }

    #[test]
    fn fix_i_dont_fell_like() {
        assert_suggestion_result(
            "But with this bug in place, I don't fell like asking the student to work with this tool",
            FeelFell::default(),
            "But with this bug in place, I don't feel like asking the student to work with this tool",
        );
    }

    #[test]
    fn fix_fell_comfortable() {
        assert_suggestion_result(
            "Technology that I fell comfortable to wok Php,Laravel, Javascript,Vue, Jquery, MySqli, sqLite.",
            FeelFell::default(),
            "Technology that I feel comfortable to wok Php,Laravel, Javascript,Vue, Jquery, MySqli, sqLite.",
        );
    }

    #[test]
    fn fix_fell_good() {
        assert_suggestion_result(
            "I've ha a touch of the flu and didn't fell good enough to mess with the computer.",
            FeelFell::default(),
            "I've ha a touch of the flu and didn't feel good enough to mess with the computer.",
        );
    }

    #[test]
    fn fix_didnt_fell() {
        assert_suggestion_result(
            "They have served me well, and I didn't fell that it's a gamble.",
            FeelFell::default(),
            "They have served me well, and I didn't feel that it's a gamble.",
        );
    }

    #[test]
    fn fix_fell_free() {
        assert_suggestion_result(
            "Please fell free to add more songs.",
            FeelFell::default(),
            "Please feel free to add more songs.",
        );
    }

    #[test]
    #[ignore = "Needs more context or better heuristics"]
    fn fix_fell_right() {
        assert_suggestion_result(
            "It may fell right first but only causes confusion in long run.",
            FeelFell::default(),
            "It may feel right first but only causes confusion in long run.",
        );
    }

    #[test]
    fn dont_flag_fell_right_into() {
        assert_no_lints(
            "I followed the instructions in the browser, and waited, then it fell right into shape, and the system is working out.",
            FeelFell::default(),
        );
    }

    #[test]
    fn dont_flag_fell_right_through() {
        assert_no_lints(
            "In this case the whole Piper context menu entry is missing since the uncaught exception fell right through the whole context menu factory.",
            FeelFell::default(),
        );
    }

    #[test]
    fn fix_does_not_fell_comfortable() {
        assert_suggestion_result(
            "she does not fell comfortable with the \" iso \"-format",
            FeelFell::default(),
            "she does not feel comfortable with the \" iso \"-format",
        );
    }

    #[test]
    #[ignore = "Needs more context or better heuristics"]
    fn dont_flag_didnt_fell_for_it() {
        assert_no_lints(
            "I even tried to trick someone else to delete and add the device but he didn't fell for it...",
            FeelFell::default(),
        );
    }

    #[test]
    fn fix_fell_that() {
        assert_suggestion_result(
            "I fell that a libSQL adapter would be a reasonable addition to the core offering.",
            FeelFell::default(),
            "I feel that a libSQL adapter would be a reasonable addition to the core offering.",
        );
    }

    #[test]
    fn fix_fell_it() {
        assert_suggestion_result(
            "I personally fell it makes the screens difficult to use",
            FeelFell::default(),
            "I personally feel it makes the screens difficult to use",
        );
    }

    #[test]
    fn fix_fell_its() {
        assert_suggestion_result(
            "but I fell it's too late to update that specific part of the API",
            FeelFell::default(),
            "but I feel it's too late to update that specific part of the API",
        );
    }

    #[test]
    fn fix_fell_im() {
        assert_suggestion_result(
            "I fell I'm missing sth and I need help.",
            FeelFell::default(),
            "I feel I'm missing sth and I need help.",
        );
    }

    #[test]
    fn fix_fell_we() {
        assert_suggestion_result(
            "i fell we will have to directly use BigDecimal here for Json encoding",
            FeelFell::default(),
            "i feel we will have to directly use BigDecimal here for Json encoding",
        );
    }

    #[test]
    fn fix_fell_i() {
        assert_suggestion_result(
            "feel free to reopen if you fell I have missed something",
            FeelFell::default(),
            "feel free to reopen if you feel I have missed something",
        );
    }

    #[test]
    fn fix_fell_you() {
        assert_suggestion_result(
            "But, I maybe fell you are stepping away from what a Markdown link actually is",
            FeelFell::default(),
            "But, I maybe feel you are stepping away from what a Markdown link actually is",
        );
    }
}
