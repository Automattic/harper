use crate::{
    linting::{Lint, LintKind, Suggestion},
    patterns::{Pattern, SequencePattern, WordSet},
    Token,
};

use crate::linting::PatternLinter;

pub struct NoContractionWithVerb {
    pattern: Box<dyn Pattern>,
}

impl Default for NoContractionWithVerb {
    fn default() -> Self {
        let pattern = SequencePattern::default()
            .then_word_set(WordSet::all(&["lets", "let"]))
            .then_whitespace()
            .then_verb();

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for NoContractionWithVerb {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
        let problem_span = matched_tokens.first().unwrap().span;
        let template = problem_span.get_content(source);

        Lint {
            span: problem_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("let's", template),
                Suggestion::replace_with_match_case_str("let us", template),
            ],
            message: "It seems you forgot to include a subject here.".to_owned(),
            priority: 31,
        }
    }

    fn description(&self) -> &'static str {
        "Make sure you include a subject when giving permission to it."
    }
}
