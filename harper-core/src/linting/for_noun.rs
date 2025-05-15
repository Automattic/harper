use crate::{
    Token,
    patterns::{NominalPhrase, OwnedPatternExt, Pattern, SequencePattern, Word},
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct ForNoun {
    pattern: Box<dyn Pattern>,
}

impl Default for ForNoun {
    fn default() -> Self {
        let pattern = SequencePattern::aco("fro")
            .then_whitespace()
            .then(NominalPhrase.or(Word::new("sure")));

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for ForNoun {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;
        let problem_chars = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "for",
                problem_chars,
            )],
            message: "`For` is more common in this context.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects the archaic or mistaken `fro` to `for` when followed by a noun."
    }
}

#[cfg(test)]
mod tests {
    use super::ForNoun;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_fro_basic_correction() {
        assert_suggestion_result(
            "I got a text fro Sarah.",
            ForNoun::default(),
            "I got a text for Sarah.",
        );
    }

    #[test]
    fn allows_for_clean() {
        assert_lint_count("I got a text for Sarah.", ForNoun::default(), 0);
    }

    #[test]
    fn corrects_fro_sure() {
        assert_suggestion_result(
            "He was away fro sure!",
            ForNoun::default(),
            "He was away for sure!",
        );
    }
}
