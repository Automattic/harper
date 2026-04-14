use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct SolveFor {
    expr: SequenceExpr,
}

impl Default for SolveFor {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["solve", "solved", "solves", "solving"])
                .t_ws()
                .t_aco("for")
                .t_ws()
                .then_word_set(&[
                    "the", "a", "an", "this", "that", "my", "your", "his", "her", "its", "our",
                    "their", "these", "those", "any", "some",
                ]),
        }
    }
}

impl ExprLinter for SolveFor {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // toks: [solve(0), ws(1), "for"(2), ws(3), article(4)]
        // Remove "for " (toks[2..4]) to turn "solve for the X" into "solve the X"
        let span = toks[2..4].span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::Remove],
            message: "Use `solve` instead of `solve for` when followed by a determiner or article. Reserve `solve for` for mathematical equations (e.g., \"solve for x\").".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags incorrect use of `solve for` followed by a determiner, suggesting removal of `for`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::SolveFor;

    #[test]
    fn fix_solve_for_the_problem() {
        assert_suggestion_result(
            "We need to solve for the problem.",
            SolveFor::default(),
            "We need to solve the problem.",
        );
    }

    #[test]
    fn fix_solve_for_this_issue() {
        assert_suggestion_result(
            "How can we solve for this issue?",
            SolveFor::default(),
            "How can we solve this issue?",
        );
    }

    #[test]
    fn fix_solved_for_the_bug() {
        assert_suggestion_result(
            "They solved for the bug quickly.",
            SolveFor::default(),
            "They solved the bug quickly.",
        );
    }

    #[test]
    fn fix_solving_for_the_bottleneck() {
        assert_suggestion_result(
            "We are solving for the bottleneck.",
            SolveFor::default(),
            "We are solving the bottleneck.",
        );
    }

    #[test]
    fn no_lint_solve_for_x() {
        assert_no_lints("Solve for x in the equation.", SolveFor::default());
    }

    #[test]
    fn no_lint_solve_for_n() {
        assert_no_lints("We need to solve for n.", SolveFor::default());
    }
}
