use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct DependsBasedOn {
    expr: SequenceExpr,
}

impl Default for DependsBasedOn {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_capitalization_of("depends")
                .t_ws()
                .t_aco("based")
                .t_ws()
                .t_aco("on"),
        }
    }
}

impl ExprLinter for DependsBasedOn {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));

        let span = matched_tokens[1..3].span()?;
        let lint_kind = LintKind::Usage;
        let suggestions = vec![Suggestion::Remove];
        let message = "The word `based` is redundant here. Prefer `depends on`.".to_string();

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
        "Removes the redundant word `based` from `depends based on`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::DependsBasedOn;

    #[test]
    fn size_features() {
        assert_suggestion_result(
            "The default size is 35 KB And it depends Based on your choice of features.",
            DependsBasedOn::default(),
            "The default size is 35 KB And it depends on your choice of features.",
        );
    }

    #[test]
    fn it_colours() {
        assert_suggestion_result(
            "It depends based on the neighboring two background colors.",
            DependsBasedOn::default(),
            "It depends on the neighboring two background colors.",
        );
    }

    #[test]
    fn dont_flag_period() {
        assert_no_lints(
            "Name of the task(s) on which the execution of this task depends. Based on the execution status of all the tasks",
            DependsBasedOn::default(),
        );
    }

    #[test]
    fn wrapping_subjective() {
        assert_suggestion_result(
            "whether or not a developer wants a chain to be wrapped/unwrapped typically depends based on a number of subjective ...",
            DependsBasedOn::default(),
            "whether or not a developer wants a chain to be wrapped/unwrapped typically depends on a number of subjective ...",
        );
    }

    #[test]
    fn answer_spec() {
        assert_suggestion_result(
            "it's entirely possible that the answer is it depends based on this part of the spec",
            DependsBasedOn::default(),
            "it's entirely possible that the answer is it depends on this part of the spec",
        );
    }

    #[test]
    fn perms_token_type() {
        assert_suggestion_result(
            "The permissions needed by gh repo-stats depends based on -y, --token-type",
            DependsBasedOn::default(),
            "The permissions needed by gh repo-stats depends on -y, --token-type",
        );
    }

    #[test]
    fn bucket_thread() {
        assert_suggestion_result(
            "My understanding is that the result of assignBucket depends based on current thread.",
            DependsBasedOn::default(),
            "My understanding is that the result of assignBucket depends on current thread.",
        );
    }

    #[test]
    fn scaling_traffic() {
        assert_suggestion_result(
            "horizontal or vertical scaling depends based on type of traffic and type of operations performed on those traffic flows",
            DependsBasedOn::default(),
            "horizontal or vertical scaling depends on type of traffic and type of operations performed on those traffic flows",
        );
    }

    #[test]
    fn it_variable() {
        assert_suggestion_result(
            "It depends based on the value of includePosts variable.",
            DependsBasedOn::default(),
            "It depends on the value of includePosts variable.",
        );
    }

    #[test]
    fn component_no() {
        assert_suggestion_result(
            "Depends based on the component, no. clearOnHide",
            DependsBasedOn::default(),
            "Depends on the component, no. clearOnHide",
        );
    }

    #[test]
    fn function_actions() {
        assert_suggestion_result(
            "The objective function depends based on \"total actions\"",
            DependsBasedOn::default(),
            "The objective function depends on \"total actions\"",
        );
    }

    #[test]
    fn it_provider() {
        assert_suggestion_result(
            "It depends based on your domain provider, but usually there's an option in their settings panel",
            DependsBasedOn::default(),
            "It depends on your domain provider, but usually there's an option in their settings panel",
        );
    }

    #[test]
    fn dont_flag_comma() {
        assert_no_lints(
            "I have realized most of it's functionalities our library depends, based on native http module",
            DependsBasedOn::default(),
        );
    }

    #[test]
    fn dont_flag_paren() {
        assert_no_lints(
            "in between : it depends (based on before or after the commits above)",
            DependsBasedOn::default(),
        );
    }
}
