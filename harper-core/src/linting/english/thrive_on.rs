use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct ThriveOn {
    expr: SequenceExpr,
}

impl Default for ThriveOn {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["thrive", "thrived", "thrives", "thriving"])
                .t_ws()
                .t_aco("off")
                .then_optional(SequenceExpr::whitespace().t_aco("of")),
        }
    }
}

impl ExprLinter for ThriveOn {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects `thrive off` and `thrive off of` to `thrive on`."
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks[2..].span()?;
        Some(Lint {
            span,
            lint_kind: LintKind::Style,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "on",
                span.get_content(src),
            )],
            message: "Consider using `thrive on` instead of `thrive off` or `thrive off of`."
                .to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ThriveOn;

    // definite mistakes

    #[test]
    fn ignore_thrive_off() {
        assert_suggestion_result(
            "I thrive off open source projects.",
            ThriveOn::default(),
            "I thrive on open source projects.",
        );
    }

    #[test]
    fn fix_thrive_off_of() {
        assert_suggestion_result(
            "I thrive off of criticism and constructive feedback.",
            ThriveOn::default(),
            "I thrive on criticism and constructive feedback.",
        );
    }

    #[test]
    fn ignore_thrived_off() {
        assert_suggestion_result(
            "The decision hurt the emulator community who thrived off AVX512 chips at the low to medium end, not for the large registers, but for the masking capabilities.",
            ThriveOn::default(),
            "The decision hurt the emulator community who thrived on AVX512 chips at the low to medium end, not for the large registers, but for the masking capabilities.",
        );
    }
    #[test]
    fn ignore_thrived_off_of() {
        assert_suggestion_result(
            "All my life I have been determined to succeed in my education and have thrived off of an overwhelming amount of creativity.",
            ThriveOn::default(),
            "All my life I have been determined to succeed in my education and have thrived on an overwhelming amount of creativity.",
        );
    }

    #[test]
    fn ignore_thrives_off_collaboration() {
        assert_suggestion_result(
            "Open source thrives off collaboration.",
            ThriveOn::default(),
            "Open source thrives on collaboration.",
        );
    }

    #[test]
    fn ignore_thrives_off_of_community() {
        assert_suggestion_result(
            "An all-in-one technical interview prep platform which fosters and thrives off of community engagement.",
            ThriveOn::default(),
            "An all-in-one technical interview prep platform which fosters and thrives on community engagement.",
        );
    }

    #[test]
    fn fix_thriving_off() {
        assert_suggestion_result(
            "So the government granted monopolies to those companies willing to risk the investment, and our teleco's have been thriving off it ever since.",
            ThriveOn::default(),
            "So the government granted monopolies to those companies willing to risk the investment, and our teleco's have been thriving on it ever since.",
        );
    }

    fn fix_thriving_off_of() {
        assert_suggestion_result(
            "Thriving off of solving problems on the wall AND in front of my computer screen",
            ThriveOn::default(),
            "Thriving on solving problems on the wall AND in front of my computer screen",
        );
    }

    // changing to "thrive on" isn't always a definite win when "thrive off" seems to mix with "live off"

    // Obviously confusing "live off the land" with "thrive on". The phrase "thrive on the land" gets no Google hits.
    #[test]
    fn ignore_thrive_off_the_land() {
        assert_suggestion_result(
            "A great collection of resources to thrive off the land.",
            ThriveOn::default(),
            "A great collection of resources to thrive off the land.",
        );
    }

    // This one definitely doesn't sound right with "thrive on his back" - but is it common enough to check for?
    #[test]
    #[ignore = "Analogy with 'live off his back'"]
    fn ignore_thriving_off_of_his_back() {
        assert_no_lints(
            "We're getting free software, that we ALL use, and the person who has worked tirelessly on it is struggling financially, while we are all thriving off of his back.",
            ThriveOn::default(),
        );
    }
}
