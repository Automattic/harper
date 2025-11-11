use crate::expr::{Expr, FixedPhrase, SequenceExpr};
use crate::linting::{ExprLinter, LintKind, Suggestion};
use crate::patterns::Word;
use crate::{Lint, Token};

pub struct MostOfTheTimes {
    expr: Box<dyn Expr>,
}

impl Default for MostOfTheTimes {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::any_of(vec![
                    Box::new(FixedPhrase::from_phrase("a lot")),
                    Box::new(Word::new("most")),
                ])
                .t_ws()
                .then_fixed_phrase("of the times"),
            ),
        }
    }
}

impl ExprLinter for MostOfTheTimes {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.last()?.span;
        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                "time".chars().collect(),
                span.get_content(src),
            )],
            message: "Singular `time` is usually the correct form in this context.".to_string(),
            priority: 32,
        })
    }

    fn description(&self) -> &str {
        "Corrects `a lot of the times` and `most of the times` to use singular `time`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::MostOfTheTimes;

    #[test]
    fn hangs_forever() {
        assert_suggestion_result(
            "restic backup hangs forever most of the times · Issue #2834",
            MostOfTheTimes::default(),
            "restic backup hangs forever most of the time · Issue #2834",
        );
    }

    #[test]
    fn options_are_ignored() {
        assert_suggestion_result(
            "but other options like device and options are ignored most of the times",
            MostOfTheTimes::default(),
            "but other options like device and options are ignored most of the time",
        );
    }

    #[test]
    fn parenthesized() {
        assert_suggestion_result(
            "prompted html code gets (most of the times) read by copilot but is not displayed.",
            MostOfTheTimes::default(),
            "prompted html code gets (most of the time) read by copilot but is not displayed.",
        );
    }

    #[test]
    fn i_cant_play() {
        assert_suggestion_result(
            "I cannot get the version 1.0 without c so I cant play a lot of the times with other people",
            MostOfTheTimes::default(),
            "I cannot get the version 1.0 without c so I cant play a lot of the time with other people",
        );
    }
}
