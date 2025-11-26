use crate::{
    Lint, Token,
    expr::{Expr, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, LintKind},
    token_string_ext::TokenStringExt,
};

pub struct ExtraneousDidPast {
    expr: Box<dyn Expr>,
}

impl Default for ExtraneousDidPast {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::longest_of(vec![
                    Box::new(SequenceExpr::word_set(&["did", "didn't", "didnt"])),
                    Box::new(FixedPhrase::from_phrase("did not")),
                    // Box::new(SequenceExpr::aco("did"))
                ])
                .t_ws()
                .then_kind_is_but_is_not_except(
                    // match "did wrote" but not "did read"
                    |k| k.is_verb_simple_past_form(),
                    |k| k.is_verb_lemma(),
                    // don't match "what I did was ..."
                    &["was"],
                ),
            ),
        }
    }
}

impl ExprLinter for ExtraneousDidPast {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        // eprintln!("💔 '{}'", toks.span()?.get_content_string(src));
        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Redundancy,
            // TODO: suggestions will involve converting past tense forms, including irregulars, to lemmas/present,
            message:
                "It's redundant to use the past form `did` and also the past form of the main verb."
                    .to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Detects the redundant use of `did` with a past tense verb (e.g., `did went`, `did not had`). Use either `did` + base form (`did go`, `did not have`) or just the past tense (`went`, `had`)."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linting::tests::{assert_lint_count, assert_no_lints};

    #[test]
    fn flag_did_had() {
        assert_lint_count(
            "I did had the version 10 and now I reverted to the 9 but still have the issue.",
            ExtraneousDidPast::default(),
            1,
        );
    }

    #[test]
    fn flag_did_thought() {
        assert_lint_count(
            "I did thought about a init-issue, because the hardware is the same which I succesfully used in older versions of RunCPM.",
            ExtraneousDidPast::default(),
            1,
        );
    }

    #[test]
    fn flag_did_not_had() {
        assert_lint_count(
            "my local backend did not had translation files",
            ExtraneousDidPast::default(),
            1,
        );
    }

    #[test]
    fn flag_did_not_knew() {
        assert_lint_count(
            "Did not knew that this default recommended OS came with an full desktop.",
            ExtraneousDidPast::default(),
            1,
        )
    }

    #[test]
    fn flag_did_not_slept() {
        assert_lint_count(
            "He did not slept yesterday.",
            ExtraneousDidPast::default(),
            1,
        );
    }

    #[test]
    fn flag_did_not_thought() {
        assert_lint_count(
            "I did not thought (when making this change) that it would cause another issue.",
            ExtraneousDidPast::default(),
            1,
        );
    }

    #[test]
    fn flag_did_not_went() {
        assert_lint_count(
            "He did not went as far as using md5sum on both files",
            ExtraneousDidPast::default(),
            1,
        );
    }

    #[test]
    fn flag_didnt_came() {
        assert_lint_count(
            "Frigate 0.16.2 didn't came up, had to switch back to 0.16.1 which works fine.",
            ExtraneousDidPast::default(),
            1,
        );
    }

    #[test]
    fn flag_didnt_came_no_apostrophe() {
        assert_lint_count(
            "But on the nRF side connection complete event didnt came.",
            ExtraneousDidPast::default(),
            1,
        );
    }
    #[test]
    fn flag_didnt_had() {
        assert_lint_count(
            "when I started this project, playwright didn't had such a util",
            ExtraneousDidPast::default(),
            1,
        );
    }

    #[test]
    fn flag_didnt_knew() {
        assert_lint_count(
            "An addon you didn't knew you needed!",
            ExtraneousDidPast::default(),
            1,
        );
    }

    #[test]
    fn dont_flag_didnt_thought() {
        assert_lint_count(
            "Wow, didn't thought about that, that should work, will try it out",
            ExtraneousDidPast::default(),
            1,
        );
    }

    // Known false positives. Please contribute PRs which address them.

    #[test]
    #[ignore = "Known false positive"]
    fn dont_flag_just_did_had_to_have_been() {
        assert_no_lints(
            "What you just did had to have been a real bitch.",
            ExtraneousDidPast::default(),
        );
    }
}
