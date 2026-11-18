use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Sentence},
};

pub struct CoolCalmCollected {
    expr: SequenceExpr,
}

impl Default for CoolCalmCollected {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("cool")
                .then_optional_comma()
                .t_ws()
                .t_aco("calm")
                .then_optional_comma()
                .t_ws()
                .t_aco("and")
                .then_optional_comma()
                .t_ws()
                .t_aco("collective"),
        }
    }
}

impl ExprLinter for CoolCalmCollected {
    type Unit = Sentence;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.last()?.span;
        Some(Lint {
            span,
            lint_kind: LintKind::Malapropism,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "collected",
                span.get_content(src),
            )],
            message: "`A `collective` is a kind of group. This idiom uses the word `collected`.`"
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the malapropism `cool, calm, and collective` to use the correct `collected`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::CoolCalmCollected;

    #[test]
    fn first_comma_only() {
        assert_suggestion_result(
            "One thing that annoys me the most is when people say “cool, calm and collective”.",
            CoolCalmCollected::default(),
            "One thing that annoys me the most is when people say “cool, calm and collected”.",
        );
    }

    #[test]
    fn both_commas() {
        assert_suggestion_result(
            "\"Cool, calm, and collective\" is apart of a list of my favorite blundered sayings.",
            CoolCalmCollected::default(),
            "\"Cool, calm, and collected\" is apart of a list of my favorite blundered sayings.",
        );
    }

    #[test]
    fn only_comma_is_in_wrong_position() {
        assert_suggestion_result(
            "Your all cool calm and, collective With your shity paddles",
            CoolCalmCollected::default(),
            "Your all cool calm and, collected With your shity paddles",
        );
    }

    #[test]
    fn no_commas() {
        assert_suggestion_result(
            "But being able to stay cool calm and collective in that situation is extremely important in this job.",
            CoolCalmCollected::default(),
            "But being able to stay cool calm and collected in that situation is extremely important in this job.",
        );
    }
}
