use crate::expr::{Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::{
    Lint, Token,
    linting::{ExprLinter, LintKind, Suggestion},
};

/// Corrects "get access at <resource>" to "get access to <resource>", while avoiding
/// false positives like "get access at all", "get access at least", URLs, or time expressions.
pub struct GetAccessAt {
    expr: SequenceExpr,
}

impl Default for GetAccessAt {
    fn default() -> Self {
        let verbs = &[
            "get",
            "gets",
            "got",
            "getting",
            "gained",
            "gaining",
            "obtain",
            "obtains",
            "obtained",
            "obtaining",
        ];

        let pattern = SequenceExpr::word_set(verbs)
            .t_ws()
            .t_aco("access")
            .t_ws()
            .t_aco("at")
            .t_ws()
            .then_word_except(&["all", "least"])
            .t_ws()
            .then_any_word();

        Self { expr: pattern }
    }
}

impl ExprLinter for GetAccessAt {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let at_tok = toks.get(4)?;

        if let Some(next) = toks.get(6) {
            let next_str = next.get_str(src).to_lowercase();

            // False positive check: URLs / Web addresses
            if next_str.starts_with("http://")
                || next_str.starts_with("https://")
                || next_str.contains("www.")
            {
                return None;
            }

            // False positive check: "at this time", "at the moment", "at the level", "at the stage"
            let is_time_or_level = matches!(next_str.as_str(), "this" | "that" | "the")
                && toks.get(8).is_some_and(|following| {
                    let fol_str = following.get_str(src).to_lowercase();
                    matches!(fol_str.as_str(), "time" | "moment" | "stage" | "level")
                });

            if is_time_or_level {
                return None;
            }
        }

        Some(Lint {
            span: at_tok.span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::ReplaceWith("to".chars().collect())],
            message: "Did you mean `to` instead of `at` when specifying what is being accessed?"
                .to_owned(),
            priority: 45,
        })
    }

    fn description(&self) -> &str {
        "Flags 'get access at' and suggests replacing 'at' with 'to'."
    }
}

#[cfg(test)]
mod tests {
    use super::GetAccessAt;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn test_get_access_at_cpu() {
        assert_suggestion_result(
            "They really need to get access at some CPU time.",
            GetAccessAt::default(),
            "They really need to get access to some CPU time.",
        );
    }

    #[test]
    fn test_get_access_at_models() {
        assert_suggestion_result(
            "In order to get access at models, send a request.",
            GetAccessAt::default(),
            "In order to get access to models, send a request.",
        );
    }

    #[test]
    fn test_got_access_at_database() {
        assert_suggestion_result(
            "I got access at the server database.",
            GetAccessAt::default(),
            "I got access to the server database.",
        );
    }

    // --- False Positive Tests ---

    #[test]
    fn ignore_access_at_all() {
        assert_lint_count(
            "I didn't manage to get access at all.",
            GetAccessAt::default(),
            0,
        );
    }

    #[test]
    fn ignore_access_at_least() {
        assert_lint_count(
            "Is it possible to get access at least to the file content?",
            GetAccessAt::default(),
            0,
        );
    }

    #[test]
    fn ignore_access_at_this_time() {
        assert_lint_count(
            "The quickest method to get access at this time is to submit a form.",
            GetAccessAt::default(),
            0,
        );
    }

    #[test]
    fn ignore_access_at_url() {
        assert_lint_count(
            "Get access at https://example.com/login",
            GetAccessAt::default(),
            0,
        );
    }
}
