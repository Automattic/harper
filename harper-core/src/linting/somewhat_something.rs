use crate::{
    patterns::{Pattern, SequencePattern},
    Token,
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct SomewhatSomething {
    pattern: Box<dyn Pattern>,
}

impl Default for SomewhatSomething {
    fn default() -> Self {
        let pattern = SequencePattern::aco("somewhat")
            .then_whitespace()
            .t_aco("of")
            .then_whitespace()
            .t_aco("a");

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for SomewhatSomething {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;
        let og = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::Style,
            suggestions: vec![Suggestion::replace_with_match_case_str("something", og)],
            message: "Use the traditional form.".to_owned(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "When describing a single instance of a noun, use `something` rather than `somewhat`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::SomewhatSomething;

    #[test]
    fn issue_414() {
        assert_suggestion_result(
            "This may be somewhat of a surprise.",
            SomewhatSomething::default(),
            "This may be something of a surprise.",
        );
    }
}
