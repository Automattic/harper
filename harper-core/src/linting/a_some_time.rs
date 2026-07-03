use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr, TimeUnitExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::Word,
};

pub struct ASomeTime {
    expr: SequenceExpr,
}

impl Default for ASomeTime {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_seq(&["a", "some"]).t_ws().then_any_of([
                Box::new(Word::new("time")) as Box<dyn Expr>,
                Box::new(TimeUnitExpr::plurals_only()),
            ]),
        }
    }
}

impl ExprLinter for ASomeTime {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], source: &[char]) -> Option<Lint> {
        let a_some_toks = &toks[0..=2];
        let a_some_span = a_some_toks.span()?;

        // We use `ReplaceWith` rather than `Remove` because the latter has no case-matching.
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "some",
            a_some_span.get_content(source),
        )];

        Some(Lint {
            span: a_some_span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "Remove the indefinite article `a` before `some`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Removes the redundant/conflicting indefinite article `a` before `some` when followed by time expressions."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::ASomeTime;

    #[test]
    fn after_a_some_time() {
        assert_suggestion_result(
            "I am connecting to a websocket and I get this error after a some time: SyntaxError: Unexpected end of JSON input",
            ASomeTime::default(),
            "I am connecting to a websocket and I get this error after some time: SyntaxError: Unexpected end of JSON input",
        );
    }

    #[test]
    fn for_a_some_months() {
        assert_suggestion_result(
            "I haven't upgraded my system for at least a some months.",
            ASomeTime::default(),
            "I haven't upgraded my system for at least some months.",
        );
    }

    #[test]
    fn for_a_some_time() {
        assert_suggestion_result(
            "The problem with this code is that not receiving images for a some time causes the window to hang",
            ASomeTime::default(),
            "The problem with this code is that not receiving images for some time causes the window to hang",
        );
    }

    #[test]
    fn for_a_some_weeks() {
        assert_suggestion_result(
            "This problem have been there for a some weeks so does it plans to ve fixed.",
            ASomeTime::default(),
            "This problem have been there for some weeks so does it plans to ve fixed.",
        );
    }

    #[test]
    fn a_some_weeks_ago() {
        assert_suggestion_result(
            "I reported some warnings from the log a some weeks ago.",
            ASomeTime::default(),
            "I reported some warnings from the log some weeks ago.",
        );
    }

    #[test]
    fn a_some_hours() {
        assert_suggestion_result(
            "I guess support people is was cheaper than just a some hours of an engineer who can actually fix the platform",
            ASomeTime::default(),
            "I guess support people is was cheaper than just some hours of an engineer who can actually fix the platform",
        );
    }

    #[test]
    fn a_some_time() {
        assert_suggestion_result(
            "Spent a some time not using it, now the problem is back....",
            ASomeTime::default(),
            "Spent some time not using it, now the problem is back....",
        );
    }
}
