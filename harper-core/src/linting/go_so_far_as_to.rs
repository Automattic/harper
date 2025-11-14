use crate::Token;
use crate::TokenStringExt;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::Suggestion;
use crate::linting::{ExprLinter, Lint, LintKind};

pub struct GoSoFarAsTo {
    exp: Box<dyn Expr>,
}

impl Default for GoSoFarAsTo {
    fn default() -> Self {
        Self {
            exp: Box::new(
                SequenceExpr::word_set(&["go", "goes", "going", "gone", "went"])
                    .then_fixed_phrase(" so far to ")
                    .then_optional(SequenceExpr::default().then_adverb().t_ws())
                    .then_any_word(),
            ),
        }
    }
}

impl ExprLinter for GoSoFarAsTo {
    fn expr(&self) -> &dyn Expr {
        self.exp.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let toks_len = toks.len();

        if toks_len != 9 && toks_len != 11 {
            return None;
        }

        let last = toks.last().unwrap();
        let penult = &toks[toks_len - 3];

        match (toks_len, penult.kind.is_adverb(), last.kind.is_verb_lemma()) {
            (11, true, true) => (),
            (9, _, true) => (),
            _ => return None,
        }

        let go_so_far_to_toks = &toks[0..=6];
        let go_so_far_to_span = go_so_far_to_toks.span()?;
        let go_so_far_toks = &toks[0..=4];
        let to_tok = &toks[6];

        let sugg = Suggestion::replace_with_match_case(
            format!(
                "{} as {}",
                go_so_far_toks.span()?.get_content_string(src),
                to_tok.span.get_content_string(src)
            )
            .chars()
            .collect(),
            go_so_far_to_span.get_content(src),
        );

        Some(Lint {
            span: go_so_far_to_span,
            lint_kind: LintKind::Nonstandard,
            suggestions: vec![sugg],
            message: "If this is intended to express going beyond what's expected, the standard idiom is `go so far as to`".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Flags 'go so far to' when it should be 'go so far as to' to express going beyond expectations"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{
        GoSoFarAsTo,
        tests::{assert_no_lints, assert_suggestion_result},
    };

    #[test]
    fn go_so_far_to() {
        assert_suggestion_result(
            "I'd even go so far to say as it's a good way to get started with getting things onto ...",
            GoSoFarAsTo::default(),
            "I'd even go so far as to say as it's a good way to get started with getting things onto ...",
        );
    }

    #[test]
    fn goes_so_far_to() {
        assert_suggestion_result(
            "I believe Java goes so far to even throw a runtime exception.",
            GoSoFarAsTo::default(),
            "I believe Java goes so far as to even throw a runtime exception.",
        );
    }

    #[test]
    fn gone_so_far_to() {
        assert_suggestion_result(
            "I've gone so far to reinstall Mac OS, which got the runner to finally start",
            GoSoFarAsTo::default(),
            "I've gone so far as to reinstall Mac OS, which got the runner to finally start",
        );
    }

    #[test]
    fn went_so_far_to() {
        assert_suggestion_result(
            "I've read these posts but only went so far to conclude that I need to potentially add sql statements into the blocks",
            GoSoFarAsTo::default(),
            "I've read these posts but only went so far as to conclude that I need to potentially add sql statements into the blocks",
        );
    }

    #[test]
    fn went_so_far_to_adverb() {
        assert_suggestion_result(
            "I even went so far to manually replace the .AppImage with a different file in the Applications folder",
            GoSoFarAsTo::default(),
            "I even went so far as to manually replace the .AppImage with a different file in the Applications folder",
        );
    }

    #[test]
    #[ignore = "A false positive we can't detect due to the next word being a verb lemma"]
    fn dont_flag_going_so_far_to() {
        assert_no_lints(
            "Why dictate that the system must be canonically described through a textual syntax – especially after going so far to make that unnecessary?",
            GoSoFarAsTo::default(),
        );
    }

    #[test]
    fn dont_flag_goes_so_far_to() {
        assert_no_lints(
            "... even so much that one line goes so far to the right",
            GoSoFarAsTo::default(),
        );
    }

    #[test]
    fn dont_flag_go_so_far_to() {
        assert_no_lints(
            "Unfortunetly, our logs don't go so far to that time, but I found something interesting.",
            GoSoFarAsTo::default(),
        );
    }
}
