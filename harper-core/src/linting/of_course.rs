use crate::{
    Token,
    linting::{Lint, LintKind, PatternLinter, Suggestion},
    patterns::{SequencePattern, WordSet},
};

pub struct OfCourse {
    pattern: Box<dyn crate::patterns::Pattern>,
}

impl Default for OfCourse {
    fn default() -> Self {
        let wrong_forms = WordSet::new(&["curse", "corse"]);
        let pattern = SequencePattern::default()
            .t_aco("of")
            .then_whitespace()
            .then(wrong_forms);

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for OfCourse {
    fn pattern(&self) -> &dyn crate::patterns::Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched: &[Token], source: &[char]) -> Option<Lint> {
        // Skip if the word before “of” is “kind” or “sort” → “kind of curse” is valid.
        if let Some(of_idx) = matched.first().map(|t| t.span.start) {
            match source.get(..of_idx).map(|src| {
                // Walk backwards over whitespace to find the preceding word token.
                let mut i = of_idx.saturating_sub(1);
                while i > 0 && src[i].is_whitespace() {
                    i -= 1;
                }
                // Return slice ending with that char to build a small string.
                let start = src[..=i]
                    .iter()
                    .rposition(|c| c.is_whitespace())
                    .map(|p| p + 1)
                    .unwrap_or(0);
                src[start..=i].iter().collect::<String>()
            }) {
                Some(prev) => {
                    let lower = prev.to_ascii_lowercase();
                    if lower == "kind" || lower == "sort" {
                        return None;
                    }
                }
                _ => (),
            }
        }

        let typo_span = matched.last()?.span;
        let original = typo_span.get_content(source);

        Some(Lint {
            span: typo_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "course".chars().collect(),
                original,
            )],
            message: "Did you mean “of **course**” (= clearly) instead of “of curse / corse”?"
                .to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Corrects the common eggcorn “of curse / corse” to “of course,” ignoring phrases like “kind of curse.”"
    }
}

#[cfg(test)]
mod tests {
    use super::OfCourse;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn flags_of_curse() {
        assert_suggestion_result("Yes, of curse!", OfCourse::default(), "Yes, of course!");
    }

    #[test]
    fn flags_of_corse() {
        assert_suggestion_result(
            "Well, of corse we can.",
            OfCourse::default(),
            "Well, of course we can.",
        );
    }

    #[test]
    fn ignores_kind_of_curse() {
        assert_lint_count("This kind of curse is dangerous.", OfCourse::default(), 0);
    }

    #[test]
    fn ignores_sort_of_curse() {
        assert_lint_count("It's a sort of curse that lingers.", OfCourse::default(), 0);
    }

    #[test]
    fn ignores_curse_of_title() {
        assert_lint_count(
            "The Curse of Strahd is a famous module.",
            OfCourse::default(),
            0,
        );
    }
}
