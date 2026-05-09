use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

/// Flags "there own", "they're own", and "theyre own" and suggests "their own".
pub struct ThereOwn {
    expr: SequenceExpr,
}

impl Default for ThereOwn {
    fn default() -> Self {
        let expr = SequenceExpr::word_set(&["there", "they're", "theyre"])
            .t_ws()
            .t_aco("own");

        Self { expr }
    }
}

impl ExprLinter for ThereOwn {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offender = matched_tokens.first()?;
        let template = offender.span.get_content(source);

        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str("their", template)],
            message: "Did you mean the possessive `their`?".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects `there own`, `they're own`, and `theyre own` to `their own`."
    }
}

#[cfg(test)]
mod tests {
    use super::ThereOwn;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_there_own() {
        assert_suggestion_result(
            "Users can split data on there own topics.",
            ThereOwn::default(),
            "Users can split data on their own topics.",
        );
    }

    #[test]
    fn corrects_theyre_own() {
        assert_suggestion_result(
            "Everybody has they're own preferences.",
            ThereOwn::default(),
            "Everybody has their own preferences.",
        );
    }

    #[test]
    fn corrects_theyre_no_apostrophe() {
        assert_suggestion_result(
            "It would be helpful for people building theyre own rockets.",
            ThereOwn::default(),
            "It would be helpful for people building their own rockets.",
        );
    }

    #[test]
    fn preserves_capitalization() {
        assert_suggestion_result(
            "There own connection pool must be configured.",
            ThereOwn::default(),
            "Their own connection pool must be configured.",
        );
    }

    #[test]
    fn does_not_flag_correct_their_own() {
        assert_lint_count(
            "They manage their own servers.",
            ThereOwn::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_there_without_own() {
        assert_lint_count(
            "Put the chairs over there by the window.",
            ThereOwn::default(),
            0,
        );
    }
}
