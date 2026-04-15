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
                .then_determiner(),
        }
    }
}

impl ExprLinter for SolveFor {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        // toks: [solve(0), ws(1), "for"(2), ws(3), article(4)]
        // Remove "for " (toks[2..4]) to turn "solve for the X" into "solve the X"
        let span = toks[2..4].span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::Remove],
            message: "\"Solve for\" is typically used in math or science contexts (e.g., \"solve for x\"). In general writing, \"solve\" alone is usually correct here.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags incorrect use of `solve for` when `solve` is correct."
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
    fn fix_solve_for_our_customers() {
        assert_suggestion_result(
            "We want to solve for our customers' needs.",
            SolveFor::default(),
            "We want to solve our customers' needs.",
        );
    }

    #[test]
    fn fix_solves_for_every_edge_case() {
        assert_suggestion_result(
            "This approach solves for every edge case.",
            SolveFor::default(),
            "This approach solves every edge case.",
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

    #[test]
    fn no_lint_solve_for_y() {
        assert_no_lints("Solve for y when x equals zero.", SolveFor::default());
    }

    #[test]
    fn fix_solve_for_a_better_solution() {
        assert_suggestion_result(
            "We need to solve for a better solution.",
            SolveFor::default(),
            "We need to solve a better solution.",
        );
    }

    #[test]
    fn fix_solve_for_the_missing_variable() {
        assert_suggestion_result(
            "I will solve for the missing variable.",
            SolveFor::default(),
            "I will solve the missing variable.",
        );
    }

    #[test]
    fn fix_solved_for_the_unknown() {
        assert_suggestion_result(
            "They solved for the unknown.",
            SolveFor::default(),
            "They solved the unknown.",
        );
    }

    #[test]
    fn fix_solves_for_this_challenge() {
        assert_suggestion_result(
            "Our team solves for this challenge every day.",
            SolveFor::default(),
            "Our team solves this challenge every day.",
        );
    }

    #[test]
    fn no_lint_solve_for_x_in_equation() {
        assert_no_lints("Can you solve for x in this equation?", SolveFor::default());
    }

    // Real-world examples from issue #613

    #[test]
    fn issue_613_solve_for_this_issue() {
        // "How can i solve for this issue with the logical flow"
        assert_suggestion_result(
            "How can I solve for this issue with the logical flow?",
            SolveFor::default(),
            "How can I solve this issue with the logical flow?",
        );
    }

    #[test]
    fn issue_613_solve_for_the_problem_email() {
        // "Can you help me solve for the problem email message"
        assert_suggestion_result(
            "Can you help me solve for the problem email message?",
            SolveFor::default(),
            "Can you help me solve the problem email message?",
        );
    }

    #[test]
    fn issue_613_solve_for_the_wrong_problem() {
        // "when you solve for the wrong problem not only do you not get the solution"
        assert_suggestion_result(
            "When you solve for the wrong problem you miss the solution.",
            SolveFor::default(),
            "When you solve the wrong problem you miss the solution.",
        );
    }

    #[test]
    fn issue_613_solve_for_this_git_problem() {
        // "this is a how to solve for this git problem: fatal: unable ..."
        assert_suggestion_result(
            "This is how to solve for this git problem.",
            SolveFor::default(),
            "This is how to solve this git problem.",
        );
    }

    #[test]
    #[ignore] // "solved for because" — no determiner after "for", linter doesn't match this pattern
    fn issue_613_solved_for_because() {
        // "this has already been solved for because the ad520 also has its own ways"
        assert_suggestion_result(
            "This has already been solved for because the ad520 has its own ways.",
            SolveFor::default(),
            "This has already been solved because the ad520 has its own ways.",
        );
    }

    #[test]
    #[ignore] // "solve for" at end of clause — no determiner follows, linter can't detect this
    fn issue_613_solve_for_end_of_clause() {
        // "the actual word that you couldn't solve for"
        assert_suggestion_result(
            "The word that you couldn't solve for.",
            SolveFor::default(),
            "The word that you couldn't solve.",
        );
    }
}
