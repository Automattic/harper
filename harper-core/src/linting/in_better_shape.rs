use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct InBetterShape {
    expr: SequenceExpr,
}

impl Default for InBetterShape {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&[
                // be
                "am", "are", "be", "been", "being", "is", "was", "were",
                // contractions of be
                "i'm", "we're", "you're", "he's", "she's", "it's", "they're",
                // contractions of be missing their apostrophes
                "im", "were", "youre", "hes", "shes", "its",
                // sense verbs behave somewhat similarly to be
                "theyre", "appears", "looks", "seems", "sounds",
                // get behaves similarly to be
                "get", "gets", "getting", "got", "gotten",
            ])
            .t_ws()
            .t_aco("in")
            .t_ws()
            .t_aco("a")
            .t_ws()
            .t_set(&["better", "worse"])
            .t_ws()
            .t_aco("shape"),
        }
    }
}

impl ExprLinter for InBetterShape {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        let span = matched_tokens[4..6].span()?;
        let lint_kind = LintKind::Usage;
        let suggestions = vec![Suggestion::Remove];
        let message =
            "If `shape` here means `fitness`, `health`, or `condition`, the `a` does not belong."
                .to_string();
        Some(Lint {
            span,
            lint_kind,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Tries to correct when 'in a shape' is should be 'in shape'."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::InBetterShape;

    #[test]
    fn dont_flag_is_a_better_shape() {
        assert_no_lints(
            "Do you think a hexagon is a better shape than a circle?",
            InBetterShape::default(),
        );
    }

    #[test]
    fn fix_we_are_in_a_better_shape() {
        assert_suggestion_result(
            "Update: thanks to the work by @HassanAbouelela and some other PRs we are in a better shape now.",
            InBetterShape::default(),
            "Update: thanks to the work by @HassanAbouelela and some other PRs we are in better shape now.",
        );
    }

    #[test]
    fn dont_flag_results_in_a_better_shape() {
        assert_no_lints(
            "Lowering the maximum iterations to below 10 results in a better shape.",
            InBetterShape::default(),
        );
    }

    #[test]
    fn fix_system_is_in_a_better_shape() {
        assert_suggestion_result(
            "but the fix is to wipe the disk properly, so now your system is in a better shape",
            InBetterShape::default(),
            "but the fix is to wipe the disk properly, so now your system is in better shape",
        );
    }

    #[test]
    fn fix_looks_in_a_better_shape() {
        assert_suggestion_result(
            "Still your code looks in a better shape :)",
            InBetterShape::default(),
            "Still your code looks in better shape :)",
        );
    }

    #[test]
    fn fix_once_its_in_a_better_shape() {
        assert_suggestion_result(
            "If you're interested, I can share my work once it's in a better shape.",
            InBetterShape::default(),
            "If you're interested, I can share my work once it's in better shape.",
        );
    }

    #[test]
    fn fix_it_is_in_a_worse_shape() {
        assert_suggestion_result(
            "preview sounds like it is in a worse shape than it actually is",
            InBetterShape::default(),
            "preview sounds like it is in worse shape than it actually is",
        );
    }

    #[test]
    fn fix_be_in_a_better_shape() {
        assert_suggestion_result(
            "here, you'd honestly be in a better shape using MobX",
            InBetterShape::default(),
            "here, you'd honestly be in better shape using MobX",
        );
    }

    #[test]
    fn fix_get_in_a_better_shape() {
        assert_suggestion_result(
            "rough experience that I hope we can get in a better shape soon",
            InBetterShape::default(),
            "rough experience that I hope we can get in better shape soon",
        );
    }

    #[test]
    fn fix_is_in_a_worse_shape() {
        assert_suggestion_result(
            "The standard library's WebSocket is in a worse shape.",
            InBetterShape::default(),
            "The standard library's WebSocket is in worse shape.",
        );
    }

    #[test]
    fn fix_was_in_a_better_shape() {
        assert_suggestion_result(
            "aaaaanndddd I thought GP was in a better shape",
            InBetterShape::default(),
            "aaaaanndddd I thought GP was in better shape",
        );
    }

    #[test]
    fn fix_is_getting_in_a_better_shape() {
        assert_suggestion_result(
            "The compiler is getting in a better shape every day and providing the low level features nicely",
            InBetterShape::default(),
            "The compiler is getting in better shape every day and providing the low level features nicely",
        );
    }
}
