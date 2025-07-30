use crate::expr::Expr;
use crate::expr::FirstMatchOf;
use crate::expr::FixedPhrase;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct OutOfDate {
    expr: Box<dyn Expr>,
}

impl Default for OutOfDate {
    fn default() -> Self {
        let pattern = FirstMatchOf::new(vec![
            Box::new(FixedPhrase::from_phrase("out of date")),
            Box::new(FixedPhrase::from_phrase("out-of date")),
            Box::new(FixedPhrase::from_phrase("out of-date")),
        ]);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for OutOfDate {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
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
