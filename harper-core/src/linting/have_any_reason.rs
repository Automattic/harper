use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct HaveAnyReason {
    expr: SequenceExpr,
}

impl Default for HaveAnyReason {
    fn default() -> Self {
        // Absolute idioms that strictly reject the plural form
        let strict_singular_verbs = [
            "believe", "suspect", "expect", "know", "think", "fear", "doubt", "lie", "live",
            "stay", "trust", "fight", "feel", "suppose",
        ];

        // Permissible only in deliberate administrative/outbound contexts
        let procedural_verbs = ["give", "offer", "change", "assign"];

        Self {
            expr: SequenceExpr::word_set(["have", "had", "has", "having"])
                .t_ws()
                .then_optional(SequenceExpr::word_set(["good", "any", "no"]).t_ws())
                .then_word_seq(&["reasons", "to"])
                .t_ws()
                .t_set(
                    strict_singular_verbs
                        .iter()
                        .chain(procedural_verbs.iter())
                        .copied(),
                ),
        }
    }
}

impl ExprLinter for HaveAnyReason {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));
        let (have_or_optional, reasons, verb) = (toks.get(2)?, toks.get_rel(-5)?, toks.last()?);
        // let got_no = toks.get(2)?.get_ch(src).eq_ch(&['n', 'o']);
        // let span = toks.get_rel(-5)?.span;
        let got_no = have_or_optional.get_ch(src).eq_ch(&['n', 'o']);
        let span = reasons.span;

        eprintln!(
            "☘️ got “no”: {}, reason span: “{}”",
            got_no,
            span.get_content_string(src)
        );

        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "reason",
            span.get_content(src),
        )];
        let message = if got_no {
            "The construction 'any reasons to' is unidiomatic here. Use the singular 'any reason to' instead."
        } else {
            "Idiomatically, native speakers prefer the singular mass-noun 'reason' when followed by psychological or belief verbs like '{}'."
        }.to_string();
        // RULE 1: Direct violation with strict internal states or baseline actions
        // "Idiomatically, native speakers prefer the singular mass-noun 'reason' when followed by psychological or belief verbs like '{}'.",

        // RULE 2: Under the strict absolute modifier "no", flag everything except clear procedural actions
        // "The construction 'any reasons to' is unidiomatic here. Use the singular 'any reason to' instead.".to_string(),
        // let message = "Fix this erorr".to_owned();
        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags unidiomatic use of the plural `reasons to` when followed by epistemic, psychological, or state verbs."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::HaveAnyReason;

    #[test]
    fn i_have_no_reasons_to_doubt() {
        assert_suggestion_result(
            "I have no reasons to doubt privacy statements but it rubs me the wrong way that I'm identifying myself by name...",
            HaveAnyReason::default(),
            "I have no reason to doubt privacy statements but it rubs me the wrong way that I'm identifying myself by name...",
        );
    }

    #[test]
    fn i_dont_have_any_reasons_to_believe() {
        assert_suggestion_result(
            "...and I don't have any reasons to believe it cannot happen again. Sadly, for my mother, she...",
            HaveAnyReason::default(),
            "...and I don't have any reason to believe it cannot happen again. Sadly, for my mother, she...",
        );
    }

    #[test]
    fn do_you_have_any_reasons_to_believe() {
        assert_suggestion_result(
            "Do you have any reasons to believe that granite is more immune to the effects of quantization than other tiny models?",
            HaveAnyReason::default(),
            "Do you have any reason to believe that granite is more immune to the effects of quantization than other tiny models?",
        );
    }

    #[test]
    fn do_you_really_have_any_reasons_to_change() {
        assert_suggestion_result(
            "Do you really have any reasons to change here to the present, to show that their names have not changed through history...",
            HaveAnyReason::default(),
            "Do you really have any reason to change here to the present, to show that their names have not changed through history...",
        );
    }

    #[test]
    fn i_have_good_reasons_to_believe() {
        assert_suggestion_result(
            "Describe the bug you encountered: I have good reasons to believe that the app caches DNS entries very aggressively...",
            HaveAnyReason::default(),
            "Describe the bug you encountered: I have good reasons to believe that the app caches DNS entries very aggressively...",
        );
    }

    #[test]
    fn we_have_reasons_to_believe_that() {
        assert_suggestion_result(
            "Same as Default but we have reasons to believe that checking it at runtime will noticeably affect program performance.",
            HaveAnyReason::default(),
            "Same as Default but we have reason to believe that checking it at runtime will noticeably affect program performance.",
        );
    }

    #[test]
    fn we_have_reasons_to_believe_its_worthwhile() {
        assert_suggestion_result(
            "Given their effort spent on curating additional instruction following data, we have reasons to believe it's worthwhile...",
            HaveAnyReason::default(),
            "Given their effort spent on curating additional instruction following data, we have reasons to believe it's worthwhile...",
        );
    }

    #[test]
    fn if_you_have_reasons_to_believe() {
        assert_suggestion_result(
            "If you have reasons to believe a resource is outdated please feel free to reach out to more senior members...",
            HaveAnyReason::default(),
            "If you have reasons to believe a resource is outdated please feel free to reach out to more senior members...",
        );
    }

    #[test]
    fn i_have_reasons_to_suspect() {
        assert_suggestion_result(
            "I have a time series for which I observe a new data point. I have reasons to suspect it to be an outlier...",
            HaveAnyReason::default(),
            "I have a time series for which I observe a new data point. I have reason to suspect it to be an outlier...",
        );
    }

    #[test]
    fn but_i_have_reasons_to_suspect() {
        assert_suggestion_result(
            "I don't know but I have reasons to believe so. Should I report this as a fraud?",
            HaveAnyReason::default(),
            "I don't know but I have reason to believe so. Should I report this as a fraud?",
        );
    }
}
