use super::{Lint, LintKind, PatternLinter};
use crate::linting::Suggestion;
use crate::patterns::{EitherPattern, FixedPhrase, Pattern, SimilarToPhrase};
use crate::{Token, TokenStringExt};

pub struct MapPhraseLinter {
    description: String,
    pattern: Box<dyn Pattern>,
    correct_forms: Vec<String>,
    message: String,
}

impl MapPhraseLinter {
    pub fn new(
        pattern: Box<dyn Pattern>,
        correct_forms: impl IntoIterator<Item = impl ToString>,
        message: impl ToString,
        description: impl ToString,
    ) -> Self {
        Self {
            description: description.to_string(),
            pattern,
            correct_forms: correct_forms.into_iter().map(|f| f.to_string()).collect(),
            message: message.to_string(),
        }
    }

    pub fn new_similar_to_phrase(phrase: &'static str, detectable_distance: u8) -> Self {
        Self::new(
            Box::new(SimilarToPhrase::from_phrase(phrase, detectable_distance)),
            [phrase],
            format!("Did you mean the phrase `{phrase}`?"),
            format!("Looks for slight improper modifications to the phrase `{phrase}`."),
        )
    }

    pub fn new_fixed_phrases(
        phrase: impl IntoIterator<Item = impl AsRef<str>>,
        correct_forms: impl IntoIterator<Item = impl ToString>,
        message: impl ToString,
        description: impl ToString,
    ) -> Self {
        let patterns = EitherPattern::new(
            phrase
                .into_iter()
                .map(|p| {
                    let pattern: Box<dyn Pattern> = Box::new(FixedPhrase::from_phrase(p.as_ref()));
                    pattern
                })
                .collect(),
        );

        Self::new(Box::new(patterns), correct_forms, message, description)
    }

    pub fn new_fixed_phrase(
        phrase: impl AsRef<str>,
        correct_forms: impl IntoIterator<Item = impl ToString>,
        message: impl ToString,
        description: impl ToString,
    ) -> Self {
        Self::new(
            Box::new(FixedPhrase::from_phrase(phrase.as_ref())),
            correct_forms,
            message,
            description,
        )
    }

    pub fn new_closed_compound(phrase: impl AsRef<str>, correct_form: impl ToString) -> Self {
        let message = format!(
            "Did you mean the closed compound `{}`?",
            correct_form.to_string()
        );

        let description = format!(
            "Looks for incorrect spacing inside the closed compound `{}`.",
            correct_form.to_string()
        );

        Self::new_fixed_phrase(phrase, [correct_form], message, description)
    }
}

impl PatternLinter for MapPhraseLinter {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let matched_text = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: self
                .correct_forms
                .iter()
                .map(|correct_form| {
                    Suggestion::replace_with_match_case(
                        correct_form.chars().collect(),
                        matched_text,
                    )
                })
                .collect(),
            message: self.message.to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        self.description.as_str()
    }
}
