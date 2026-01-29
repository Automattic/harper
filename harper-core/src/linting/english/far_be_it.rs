use crate::char_string::CharStringExt;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::english::{ExprLinter, Lint, LintKind, Suggestion};
use crate::token::Token;

pub struct FarBeIt {
    expr: Box<dyn Expr>,
}

impl Default for FarBeIt {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::default()
                    .t_aco("far")
                    .t_ws()
                    .t_aco("be")
                    .t_ws()
                    .t_aco("it")
                    .t_ws()
                    .then_word_except(&["from"]),
            ),
        }
    }
}

impl ExprLinter for FarBeIt {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.last()?.span;
        let content = span.get_content(src);

        // We can only correct using `far be it for`, otherwise we recommend rephrasing the sentence.
        let (suggestions, message) = if span.get_content(src).eq_ignore_ascii_case_str("for") {
            (
                vec![Suggestion::replace_with_match_case(
                    vec!['f', 'r', 'o', 'm'],
                    content,
                )],
                "`Far be it for` is a common error for `far be it from`".to_string(),
            )
        } else {
            (vec![], "The correct usage of the idiom is `far be it from` [someone] to [do something]. Try to rephrase the sentence.".to_string())
        };

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Flags misuse of `far be it` and suggests using `from` when it is followed by `for`"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::english::FarBeIt;
    use crate::linting::tests::{
        assert_no_lints, assert_suggestion_count, assert_suggestion_result,
    };

    #[test]
    fn far_be_it_for_me_capitalized() {
        assert_suggestion_result(
            "Far be it for me to suggestion that additional cardinality be added to the already TOO MUCH CARDINALITY metric space.",
            FarBeIt::default(),
            "Far be it from me to suggestion that additional cardinality be added to the already TOO MUCH CARDINALITY metric space.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn far_be_it_for_me_lowercase() {
        assert_suggestion_result(
            "Far be it for me to tell people what to do so I'm not earnestly proposing to take away the ability to add literals to lazyframes.",
            FarBeIt::default(),
            "Far be it from me to tell people what to do so I'm not earnestly proposing to take away the ability to add literals to lazyframes.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn far_be_it_that() {
        assert_suggestion_count(
            "Far be it that I get in the middle of this thread (and the complexity WebAuthn has spawned)",
            FarBeIt::default(),
            0,
        );
    }

    #[test]
    fn far_be_it_for_the_software() {
        assert_suggestion_result(
            "Far be it for the software to give any indication of that fact.",
            FarBeIt::default(),
            "Far be it from the software to give any indication of that fact.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    #[ignore = "No punctuation between '... so far' and 'be it ...'"]
    fn missing_punctuation_false_positive() {
        assert_no_lints(
            "but it is failing for master and all the 11.x branches i have tried so far be it 11.0.0, 11.0.1 ...",
            FarBeIt::default(),
        );
    }

    #[test]
    fn far_be_it_to() {
        assert_suggestion_count(
            "I'm not a marketing guy, so far be it to second guess that.",
            FarBeIt::default(),
            0,
        );
    }
}
