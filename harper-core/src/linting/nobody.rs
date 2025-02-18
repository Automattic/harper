use crate::{
    patterns::{Pattern, SequencePattern},
    Token, TokenStringExt,
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct Nobody {
    pattern: Box<dyn Pattern>,
}

impl Default for Nobody {
    fn default() -> Self {
        let pattern = SequencePattern::aco("no")
            .then_whitespace()
            .t_aco("body")
            .then_whitespace()
            .then_verb();
        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for Nobody {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens[0..3].span()?;
        let orig_chars = span.get_content(source);
        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "nobody".chars().collect(),
                orig_chars,
            )],
            message: format!("Did you mean the closed compound `{}`?", "nobody"),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Looks for incorrect spacing inside the closed compound `nobody`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Nobody;

    #[test]
    fn both_valid_and_invalid() {
        assert_suggestion_result(
            "No body told me. I have a head but no body.",
            Nobody::default(),
            "Nobody told me. I have a head but no body.",
        );
    }
}
