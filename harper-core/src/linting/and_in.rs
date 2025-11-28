use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};

pub struct AndIn {
    expr: Box<dyn Expr>,
}

impl Default for AndIn {
    fn default() -> Self {
        Self {
            expr: Box::new(SequenceExpr::fixed_phrase("an in").then_optional_hyphen()),
        }
    }
}

impl ExprLinter for AndIn {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &*self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() != 3 {
            return None;
        }

        Some(Lint {
            span: toks[0].span,
            lint_kind: LintKind::Typo,
            message: "Did you mean `and in`?".to_string(),
            suggestions: vec![Suggestion::replace_with_match_case(
                ['a', 'n', 'd'].to_vec(),
                toks[2].span.get_content(src),
            )],
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Fixes the incorrect phrase `an in` to `and in` for proper conjunction usage."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{
        AndIn,
        tests::{assert_no_lints, assert_suggestion_result},
    };

    #[test]
    fn dont_flag_an_in_house() {
        assert_no_lints(
            "for several years as an in-house engine, used to ...",
            AndIn::default(),
        );
    }

    #[test]
    fn dont_flag_an_in_memory() {
        assert_no_lints(
            "including an in-memory real-time Vector Index,",
            AndIn::default(),
        );
    }

    #[test]
    fn dont_flag_an_in_the_moment() {
        assert_no_lints(
            "His words serve as an in-the-moment explanation for what had happened.",
            AndIn::default(),
        );
    }

    #[test]
    fn fix_an_in_to_and_in() {
        assert_suggestion_result(
            "This is an expensive operation, so try to only do it at startup an in tests.",
            AndIn::default(),
            "This is an expensive operation, so try to only do it at startup and in tests.",
        );
    }

    #[test]
    #[ignore = "This is a known false positive - `an in` can be valid in some contexts"]
    fn dont_flag_an_in_with_company() {
        assert_no_lints(
            "His parents got him an in with the company.",
            AndIn::default(),
        );
    }
}
