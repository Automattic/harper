use crate::{
    Token,
    linting::{Lint, LintKind, PatternLinter, Suggestion},
    patterns::{Pattern, SequencePattern, WordSet},
};

pub struct ItsContraction {
    pattern: Box<dyn Pattern>,
}

impl Default for ItsContraction {
    fn default() -> Self {
        let its = WordSet::new(&["its"]);
        let verbs = WordSet::new(&["had", "been", "got"]);
        let pattern = SequencePattern::default()
            .then(its)
            .then_whitespace()
            .then(verbs);
        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for ItsContraction {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], source: &[char]) -> Option<Lint> {
        let offender = toks.first()?;
        let offender_chars = offender.span.get_content(source);
        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("it's", offender_chars),
                Suggestion::replace_with_match_case_str("it has", offender_chars),
            ],
            message: "Use `it's` (short for `it has`) here, not the possessive `its`.".to_owned(),
            priority: 54,
        })
    }

    fn description(&self) -> &str {
        "Detects the possessive `its` before `had`, `been`, or `got` and offers `it's` or `it has`."
    }
}

#[cfg(test)]
mod tests {
    use super::ItsContraction;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fix_had() {
        assert_suggestion_result(
            "Its had an enormous effect.",
            ItsContraction::default(),
            "It's had an enormous effect.",
        );
    }

    #[test]
    fn fix_been() {
        assert_suggestion_result(
            "Its been months since we spoke.",
            ItsContraction::default(),
            "It's been months since we spoke.",
        );
    }

    #[test]
    fn fix_got() {
        assert_suggestion_result(
            "I think its got nothing to do with us.",
            ItsContraction::default(),
            "I think it's got nothing to do with us.",
        );
    }

    #[test]
    fn ignore_correct_contraction() {
        assert_lint_count(
            "It's been a long year for everyone.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn ignore_possessive() {
        assert_lint_count(
            "The company revised its policies last week.",
            ItsContraction::default(),
            0,
        );
    }
}
