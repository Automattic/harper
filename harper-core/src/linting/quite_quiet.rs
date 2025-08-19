use crate::expr::{Expr, FirstMatchOf, FixedPhrase, SequenceExpr};
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};
use crate::{Token, TokenKind, TokenStringExt};

pub struct QuiteQuiet {
    expr: Box<dyn Expr>,
}

impl Default for QuiteQuiet {
    fn default() -> Self {
        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                Box::new(
                    SequenceExpr::default()
                        .then(FixedPhrase::from_phrase("quiet "))
                        .then_kind_any_except(
                            &[TokenKind::is_adjective, TokenKind::is_adverb] as &[_],
                            &["here", "up"],
                        ),
                ),
                Box::new(
                    SequenceExpr::default()
                        .then_kind_except(TokenKind::is_adverb, &["never"])
                        .then(FixedPhrase::from_phrase(" quite")),
                ),
            ])),
        }
    }
}

impl ExprLinter for QuiteQuiet {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let text = toks.span()?.get_content_string(src).to_lowercase();

        if text.ends_with(" quite") {
            let first_token = toks.first()?;
            let quite_span = toks.last()?.span;

            return Some(Lint {
                span: quite_span,
                lint_kind: LintKind::Typo,
                suggestions: vec![Suggestion::replace_with_match_case(
                    "quiet".chars().collect(),
                    quite_span.get_content(src),
                )],
                message: "After an adverb like '{}', use 'quiet' not 'quite'"
                    .replace("{}", &first_token.span.get_content_string(src)),
                priority: 63,
            });
        } else if text.starts_with("quiet ") {
            let last_token = toks.last()?;
            let quiet_span = toks.first()?.span;

            return Some(Lint {
                span: quiet_span,
                lint_kind: LintKind::Typo,
                suggestions: vec![Suggestion::replace_with_match_case(
                    "quite".chars().collect(),
                    quiet_span.get_content(src),
                )],
                message: format!(
                    "Before an adjective or adverb like '{}', use 'quite' not 'quiet'",
                    last_token.span.get_content_string(src)
                ),
                priority: 63,
            });
        }

        None
    }

    fn description(&self) -> &str {
        "Corrects when `quiet` is a typo for `quite` or the other way around."
    }
}

#[cfg(test)]
mod tests {
    use super::QuiteQuiet;
    use crate::linting::tests::assert_suggestion_result;

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
}
