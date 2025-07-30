use crate::{
    Token,
    linting::{Lint, LintKind, PatternLinter, Suggestion},
    patterns::{Pattern, SequencePattern, WordSet},
    word_metadata::Person,
};

pub struct NominalWants {
    pattern: Box<dyn Pattern>,
}

impl Default for NominalWants {
    fn default() -> Self {
        let miss = WordSet::new(&["wont", "wonts", "want", "wants"]);
        let pattern = SequencePattern::default()
            .then_pronoun()
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

    fn match_to_lint(&self, toks: &[Token], source: &[char]) -> Option<Lint> {
        let subject = toks.first()?;
        let offender = toks.last()?;

        let plural = subject.kind.is_plural_nominal();

        let person = subject
            .kind
            .as_word()
            .unwrap()
            .clone()
            .unwrap()
            .pronoun
            .and_then(|p| p.person)
            .unwrap_or(Person::Third);

        let replacement = if person == Person::Third {
            if plural { "want" } else { "wants" }
        } else {
            "want"
        };

        let replacement_chars: Vec<char> = replacement.chars().collect();

        if offender.span.get_content(source) == replacement_chars.as_slice() {
            return None;
        }

        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith(replacement_chars)],
            message: format!("Did you mean `{replacement}`?"),
            priority: 55,
        })
    }

    fn description(&self) -> &str {
        "Ensures you use the correct `want` / `wants` after a nominal."
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
    fn fixes_i_wont() {
        assert_suggestion_result(
            "I wonts to leave early.",
            NominalWants::default(),
            "I want to leave early.",
        );
    }

    #[test]
    fn allows_you_want() {
        assert_lint_count("What size do you want to be?", NominalWants::default(), 0);
    }

    #[test]
    fn fixes_you_wants() {
        assert_suggestion_result(
            "What do you wants?",
            NominalWants::default(),
            "What do you want?",
        );
    }

    #[test]
    fn ignores_correct_usage_they() {
        assert_lint_count("They want to help.", NominalWants::default(), 0);
    }

    #[test]
    fn ignores_correct_usage_he() {
        assert_lint_count("He wants to help.", NominalWants::default(), 0);
    }
}
