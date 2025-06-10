use crate::{
    Token, TokenStringExt,
    patterns::{EitherPattern, FixedPhrase, Pattern},
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct OutOfDate {
    pattern: Box<dyn Pattern>,
}

impl Default for OutOfDate {
    fn default() -> Self {
        let pattern = EitherPattern::new(vec![
            Box::new(FixedPhrase::from_phrase("out of date")),
            Box::new(FixedPhrase::from_phrase("out-of date")),
            Box::new(FixedPhrase::from_phrase("out of-date")),
        ]);

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for OutOfDate {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let problem_text = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case(
                "out-of-date".chars().collect(),
                problem_text,
            )],
            message: "Did you mean the compound adjective?".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Ensures that the phrase `out of date` is written with a hyphen as `out-of-date` when used as a compound adjective."
    }
}

#[cfg(test)]
mod tests {
    use super::OutOfDate;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn corrects_out_of_date() {
        assert_suggestion_result(
            "The software is out of date.",
            OutOfDate::default(),
            "The software is out-of-date.",
        );
    }

    #[test]
    fn corrects_out_of_date_with_variation() {
        assert_suggestion_result(
            "This information is out of-date.",
            OutOfDate::default(),
            "This information is out-of-date.",
        );
    }

    #[test]
    fn allows_correct_usage() {
        assert_suggestion_result(
            "The guidelines are out-of-date.",
            OutOfDate::default(),
            "The guidelines are out-of-date.",
        );
    }
}
