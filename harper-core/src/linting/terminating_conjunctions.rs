use super::{Lint, LintKind, PatternLinter};
use crate::patterns::{ConsumesRemainingPattern, Pattern, SequencePattern, WordSet};

pub struct TerminatingConjunctions {
    pattern: Box<dyn Pattern>,
}

impl Default for TerminatingConjunctions {
    fn default() -> Self {
        Self {
            pattern: Box::new(ConsumesRemainingPattern::new(Box::new(
                SequencePattern::default()
                    .then_anything_but_hyphen()
                    .then(WordSet::new(&[
                        "although",
                        "as",
                        "because",
                        "if",
                        "lest",
                        "once",
                        "only",
                        "since",
                        "supposing",
                        "than",
                        "though",
                        "till",
                        "unless",
                        "until",
                        "when",
                        "whenever",
                        "where",
                        "whereas",
                        "wherever",
                        "whether",
                        "while",
                        "or",
                        "nor",
                        "and",
                    ]))
                    .then_comma(),
            ))),
        }
    }
}

impl PatternLinter for TerminatingConjunctions {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[crate::Token], source: &[char]) -> Option<Lint> {
        let word_span = matched_tokens[1].span;
        let word = word_span.get_content_string(source);

        Some(Lint {
            span: word_span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![],
            message: format!(
                "Subordinating conjunctions like “{word}” should not appear at the end of a \
                 clause."
            ),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Subordinating conjunctions are words that create a grammatical space for another idea or clause. As such, they should never appear at the end of a clause."
    }
}

#[cfg(test)]
mod tests {
    use super::TerminatingConjunctions;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn issue_131() {
        assert_lint_count(
            "More often than, we cannot foresee that of our community.",
            TerminatingConjunctions::default(),
            1,
        )
    }

    #[test]
    fn no_false_positive() {
        assert_lint_count("Cookies and milk.", TerminatingConjunctions::default(), 0)
    }

    #[test]
    fn issue_341() {
        assert_lint_count(
            "The structure has a couple of fields marked read-only, like A and B",
            TerminatingConjunctions::default(),
            0,
        );
    }
}
