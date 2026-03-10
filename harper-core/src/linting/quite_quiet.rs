use crate::expr::{Expr, FirstMatchOf, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};
use crate::{CharStringExt, Token, TokenKind, TokenStringExt};

pub struct QuiteQuiet {
    expr: Box<dyn Expr>,
}

impl Default for QuiteQuiet {
    fn default() -> Self {
        let quiet_word = SequenceExpr::default()
            .t_aco("quiet")
            .t_ws()
            .then_kind_any_but_not_except(
                &[
                    TokenKind::is_adjective,
                    TokenKind::is_adverb,
                    TokenKind::is_verb,
                ] as &[_],
                TokenKind::is_noun,
                &["here", "up"],
            );

        let negative_contraction_quiet = SequenceExpr::with(|tok: &Token, src: &[char]| {
            if !tok.kind.is_verb() || !tok.kind.is_apostrophized() {
                return false;
            }
            tok.span
                .get_content(src)
                .ends_with_any_ignore_ascii_case_chars(&[&['n', '\'', 't'], &['n', '’', 't']])
        })
        .t_ws()
        .t_aco("quiet");

        let adverb_quite = SequenceExpr::default()
            .then_kind_except(
                TokenKind::is_adverb,
                &["actually", "never", "not", "really", "generally"],
            )
            .t_ws()
            .t_aco("quite");

        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                Box::new(quiet_word),
                Box::new(negative_contraction_quiet),
                Box::new(adverb_quite),
            ])),
        }
    }
}

impl ExprLinter for QuiteQuiet {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let text = toks.span()?.get_content_string(src).to_lowercase();

        if text.ends_with("quite") {
            let quite_span = toks.last()?.span;

            return Some(Lint {
                span: quite_span,
                lint_kind: LintKind::Typo,
                suggestions: vec![Suggestion::replace_with_match_case(
                    "quiet".chars().collect(),
                    quite_span.get_content(src),
                )],
                message: "‘Quite’ might be a typo here. It means ‘rather’ but you might be trying to say ‘quiet’ (not noisy).".to_string(),
                priority: 63,
            });
        } else if text.starts_with("quiet") {
            let quiet_span = toks.first()?.span;

            return Some(Lint {
                span: quiet_span,
                lint_kind: LintKind::Typo,
                suggestions: vec![Suggestion::replace_with_match_case(
                    "quite".chars().collect(),
                    quiet_span.get_content(src),
                )],
                message: "‘Quiet’ might be a typo here. It means ‘not noisy’ but you might be trying to say ‘quite’ (rather).".to_string(),
                priority: 63,
            });
        } else if text.ends_with("quiet") {
            let quiet_span = toks.last()?.span;

            return Some(Lint {
                span: quiet_span,
                lint_kind: LintKind::Typo,
                suggestions: vec![Suggestion::replace_with_match_case(
                    "quite".chars().collect(),
                    quiet_span.get_content(src),
                )],
                message: "‘Quiet’ might be a typo here. It means ‘not noisy’ but you might be trying to say ‘quite’ (rather).".to_string(),
                priority: 63,
            });
        }

        None
    }

    fn description(&self) -> &str {
        "Helps distinguish between ‘quiet’ (making ‘little noise’) and ‘quite’ (meaning ‘rather’)."
    }
}

#[cfg(test)]
mod tests {
    use super::QuiteQuiet;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn fix_quiet_adverb() {
        assert_suggestion_result(
            "Rendering videos 145 frames, with lightx loras for 2.1 i experience reboots quiet often.",
            QuiteQuiet::default(),
            "Rendering videos 145 frames, with lightx loras for 2.1 i experience reboots quite often.",
        );
    }

    #[test]
    fn fix_quiet_adjective() {
        assert_suggestion_result(
            "... has been already reported multiple times and I find it quiet dumb that it still exists",
            QuiteQuiet::default(),
            "... has been already reported multiple times and I find it quite dumb that it still exists",
        );
    }

    #[test]
    fn fix_very_quite() {
        assert_suggestion_result(
            "It's very quite here at night.",
            QuiteQuiet::default(),
            "It's very quiet here at night.",
        );
    }

    #[test]
    fn fix_doesnt_quiet() {
        assert_suggestion_result("doesn't quiet", QuiteQuiet::default(), "doesn't quite");
    }

    #[test]
    fn fix_doesnt_quiet_typographical_apostrophe() {
        assert_suggestion_result("doesn’t quiet", QuiteQuiet::default(), "doesn’t quite");
    }

    #[test]
    fn fix_doesnt_quiet_in_context() {
        assert_suggestion_result(
            "When we got the car back into the workshop, we actually managed to get it running and driving, but it doesn't quiet run right, and doesn't really let me rev it.",
            QuiteQuiet::default(),
            "When we got the car back into the workshop, we actually managed to get it running and driving, but it doesn't quite run right, and doesn't really let me rev it.",
        );
    }

    #[test]
    fn dont_flag_quiet_light() {
        assert_lint_count("The quiet lights in the houses", QuiteQuiet::default(), 0);
    }

    #[test]
    fn dont_flag_quiet_till() {
        assert_lint_count(
            "You’d better try and sit quiet till morning.",
            QuiteQuiet::default(),
            0,
        );
    }

    #[test]
    fn fix_cant_quiet() {
        assert_suggestion_result(
            "I can't quiet read it",
            QuiteQuiet::default(),
            "I can't quite read it",
        );
    }

    #[test]
    fn fix_wont_quiet() {
        assert_suggestion_result(
            "It won't quiet fit",
            QuiteQuiet::default(),
            "It won't quite fit",
        );
    }

    #[test]
    fn fix_couldnt_quiet() {
        assert_suggestion_result(
            "I couldn't quiet understand everything",
            QuiteQuiet::default(),
            "I couldn't quite understand everything",
        );
    }

    #[test]
    fn fix_but_its_not_quite_clear_1956() {
        assert_no_lints("But it's not quite clear", QuiteQuiet::default());
    }

    #[test]
    fn dont_flag_adv_quite_1971() {
        assert_no_lints(
            "It’s actually quite smart. It’s really quite smart. The proof is actually quite neat. Actually really quite simple. It’s actually quite strong. The Sneetches got really quite smart on that day.",
            QuiteQuiet::default(),
        );
    }

    #[test]
    fn issue_2003() {
        assert_no_lints(
            "The namespaces are generally quite short",
            QuiteQuiet::default(),
        );
    }
}
