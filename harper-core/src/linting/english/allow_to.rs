use crate::expr::{Expr, SequenceExpr};
use crate::linting::english::{ExprLinter, Lint, LintKind};
use crate::token::Token;
use crate::token_string_ext::TokenStringExt;

pub struct AllowTo {
    exp: Box<dyn Expr>,
}

impl Default for AllowTo {
    fn default() -> Self {
        Self {
            // Note: Does not include "allowed to", which is a legitimate usage in its own right.
            exp: Box::new(
                SequenceExpr::word_set(&["allow", "allowing", "allows"])
                    .t_ws()
                    .t_aco("to")
                    .then_optional(SequenceExpr::default().t_ws().then_adverb())
                    .t_ws()
                    .then_any_word(),
            ),
        }
    }
}

impl ExprLinter for AllowTo {
    fn expr(&self) -> &dyn Expr {
        self.exp.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let span = toks.span()?;
        let first = toks.first()?;
        let allow = first.span.get_content_string(_src);

        let message = format!(
            "For correct usage, either add a subject between `{allow}` and `to` (e.g., `{allow} someone to do`) or use the present participle (e.g., `{allow} doing`)."
        );

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![],
            message,
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Flags erroneous usage of `allow to` without a subject."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::english::{
        AllowTo,
        tests::{assert_lint_count, assert_no_lints},
    };

    #[test]
    fn flag_allow_to() {
        assert_lint_count(
            "Allow to change approval policy during running task # 4394.",
            AllowTo::default(),
            1,
        );
    }

    #[test]
    fn flag_allowing_to() {
        assert_lint_count(
            "Allowing to have multiple views with different filtering # 952.",
            AllowTo::default(),
            1,
        );
    }

    #[test]
    fn flag_allows_to() {
        assert_lint_count(
            "It is easily doable for classic IHostBuilder, because its extension allows to pass configure action",
            AllowTo::default(),
            1,
        );
    }

    #[test]
    fn dont_flag_allowed_to() {
        assert_no_lints(
            "In C and C++ aliasing has to do with what expression types we are allowed to access stored values through.",
            AllowTo::default(),
        );
    }

    #[test]
    fn dont_flag_allow_pronoun_to() {
        assert_no_lints(
            "It would be really great to allow me to enter body data using multipart form",
            AllowTo::default(),
        );
    }

    #[test]
    fn dont_flag_allow_noun_to() {
        assert_no_lints(
            "Allows users to export SMART statistics from any connected hard drive",
            AllowTo::default(),
        );
    }

    #[test]
    fn dont_flag_allow_np_to() {
        assert_no_lints(
            "This vulnerability allows an authenticated attacker to infer data from the database by measuring response times",
            AllowTo::default(),
        );
    }
}
