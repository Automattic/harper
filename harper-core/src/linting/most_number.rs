use crate::{
    Token, TokenStringExt,
    patterns::{All, Pattern, SequencePattern, WordSet},
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct MostNumber {
    pattern: Box<dyn Pattern>,
}

impl Default for MostNumber {
    fn default() -> Self {
        Self {
            pattern: Box::new(All::new(vec![
                // Main pattern
                Box::new(
                    SequencePattern::default()
                        .t_aco("most")
                        .t_ws()
                        .then(WordSet::new(&["amount", "number"])),
                ),
                // Context pattern
                Box::new(
                    SequencePattern::default()
                        .then_anything()
                        .then_anything()
                        .then_anything()
                        .then_anything()
                        .t_aco("of"),
                ),
            ])),
        }
    }
}

impl PatternLinter for MostNumber {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], source: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks[0..3].span()?,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case(
                format!("highest {}", toks[2].span.get_content_string(source))
                    .chars()
                    .collect(),
                toks[0..3].span()?.get_content(source),
            )],
            message: format!(
                "Did you mean `highest {}`?",
                toks[2].span.get_content_string(source)
            ),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Corrects `most number` and `most amount`"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::MostNumber;

    #[test]
    fn corrects_most_number() {
        assert_suggestion_result(
            "Find artists that have been on Spotify the most number of times.",
            MostNumber::default(),
            "Find artists that have been on Spotify the highest number of times.",
        );
    }

    #[test]
    fn corrects_most_amount() {
        assert_suggestion_result(
            "In a section of typing output results, the hisat-genotype tool has default parameters to show up the top ten alleles that the most number of reads are mapped to or compatible with.",
            MostNumber::default(),
            "In a section of typing output results, the hisat-genotype tool has default parameters to show up the top ten alleles that the highest number of reads are mapped to or compatible with.",
        );
    }

    #[test]
    fn dont_correct_most_number_without_context() {
        assert_lint_count(
            "The random non-sequential nature should prevent most number gaming/sniping/lunging.",
            MostNumber::default(),
            0,
        );
    }
}
