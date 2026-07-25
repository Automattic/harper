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
            .t_aco("at");

        Self { expr: pattern }
    }
}

impl ExprLinter for GetAccessAt {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let at_tok = toks.last()?;
        let rest: String = src.get(at_tok.span.end..)?.iter().collect();
        let rest_lower = rest.to_lowercase();

        // False positive checks: "at all", "at least"
        if rest_lower.starts_with(" all") || rest_lower.starts_with(" least") {
            return None;
        }

        // False positive check: URLs / Web addresses
        if rest_lower.starts_with(" http://")
            || rest_lower.starts_with(" https://")
            || rest_lower.starts_with(" www.")
        {
            return None;
        }

        // False positive check: "at this time", "at the moment", "at the level", "at the stage", etc.
        if rest_lower.starts_with(" this time")
            || rest_lower.starts_with(" this moment")
            || rest_lower.starts_with(" this stage")
            || rest_lower.starts_with(" this level")
            || rest_lower.starts_with(" that time")
            || rest_lower.starts_with(" that moment")
            || rest_lower.starts_with(" that stage")
            || rest_lower.starts_with(" that level")
            || rest_lower.starts_with(" the moment")
            || rest_lower.starts_with(" the stage")
            || rest_lower.starts_with(" the level")
        {
            return None;
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
