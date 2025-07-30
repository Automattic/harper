use crate::{
    Token,
    linting::{Lint, LintKind, PatternLinter, Suggestion},
    patterns::{Pattern, SequencePattern, WordSet},
};

pub struct NominalWants {
    pattern: Box<dyn Pattern>,
}

impl Default for NominalWants {
    fn default() -> Self {
        let miss = WordSet::new(&["wont", "wonts"]);
        let pattern = SequencePattern::default()
            .then_nominal()
            .then_whitespace()
            .then(miss);
        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for NominalWants {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let offender = toks.last()?;
        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith("wants".chars().collect())],
            message: "Did you mean `wants`?".to_owned(),
            priority: 55,
        })
    }

    fn description(&self) -> &str {
        "Replaces the typo `wont`/`wonts` after a nominal."
    }
}

#[cfg(test)]
mod tests {
    use super::NominalWants;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fixes_he_wonts() {
        assert_suggestion_result(
            "He wonts to join us.",
            NominalWants::default(),
            "He wants to join us.",
        );
    }

    #[test]
    fn fixes_it_wont() {
        assert_suggestion_result(
            "It wont to move forward.",
            NominalWants::default(),
            "It wants to move forward.",
        );
    }

    #[test]
    fn fixes_she_wont() {
        assert_suggestion_result(
            "She wont to leave early.",
            NominalWants::default(),
            "She wants to leave early.",
        );
    }

    #[test]
    fn ignores_correct_usage() {
        assert_lint_count("He wants to help.", NominalWants::default(), 0);
    }
}
