use crate::expr::Expr;
use crate::weir::weir_expr_to_expr;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct ChockFull {
    expr: Box<dyn Expr>,
}

impl Default for ChockFull {
    fn default() -> Self {
        Self {
            expr: weir_expr_to_expr("[chalk, choke, chocked, chucked, choked][( ), -]full")
                .unwrap(),
        }
    }
}

impl ExprLinter for ChockFull {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_toks: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_toks.span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "chock-full",
                span.get_content(source),
            )],
            message: format!(
                "The standard term is \"chock-full\"{}.",
                if matched_toks[1].kind.is_whitespace() {
                    ", and it should be hyphenated"
                } else {
                    ""
                }
            ),
            priority: 126,
        })
    }

    fn description(&self) -> &'static str {
        "Flags common soundalikes of \"chock-full\" and makes sure they're hyphenated."
    }
}

#[cfg(test)]
mod tests {
    use super::ChockFull;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn allows_correct_form() {
        assert_lint_count(
            "'Chalk full', 'chalk-full', 'choke full', and 'choke-full' are nonstandard forms of 'chock-full'.",
            ChockFull::default(),
            4,
        );
    }

    #[test]
    fn lower_space_chalk() {
        assert_suggestion_result(
            "The codebase is chalk full of errors that we need to address.",
            ChockFull::default(),
            "The codebase is chock-full of errors that we need to address.",
        );
    }

    #[test]
    fn lower_space_choke() {
        assert_suggestion_result(
            "The project is choke full of questionable decisions that we need to revisit.",
            ChockFull::default(),
            "The project is chock-full of questionable decisions that we need to revisit.",
        );
    }

    #[test]
    fn upper_space_chalk() {
        assert_suggestion_result(
            "Chalk full of deprecated methods; we should refactor.",
            ChockFull::default(),
            "Chock-full of deprecated methods; we should refactor.",
        );
    }

    #[test]
    fn upper_space_choke() {
        assert_suggestion_result(
            "Choke full of unnecessary complexity; simplify it.",
            ChockFull::default(),
            "Chock-full of unnecessary complexity; simplify it.",
        );
    }

    #[test]
    fn lower_hyphen_chalk() {
        assert_suggestion_result(
            "The code is chalk-full of bugs; we need to debug before release.",
            ChockFull::default(),
            "The code is chock-full of bugs; we need to debug before release.",
        );
    }

    #[test]
    fn lower_hyphen_choke() {
        assert_suggestion_result(
            "The project is choke-full of warnings; we should address them.",
            ChockFull::default(),
            "The project is chock-full of warnings; we should address them.",
        );
    }

    #[test]
    fn upper_hyphen_chalk() {
        assert_suggestion_result(
            "Chalk-full of features, but we only need a few.",
            ChockFull::default(),
            "Chock-full of features, but we only need a few.",
        );
    }

    #[test]
    fn upper_hyphen_choke() {
        assert_suggestion_result(
            "Choke-full of pitfalls; let's consider alternatives.",
            ChockFull::default(),
            "Chock-full of pitfalls; let's consider alternatives.",
        );
    }

    #[test]
    fn lower_space_chocked() {
        assert_suggestion_result(
            "The computer was chocked full with 64KB general use RAM (expandable up to 256KB)",
            ChockFull::default(),
            "The computer was chock-full with 64KB general use RAM (expandable up to 256KB)",
        );
    }

    #[test]
    fn lower_space_chucked() {
        assert_suggestion_result(
            "The Mandalorian is chucked full of references and callbacks, mostly to previous Star Wars content.",
            ChockFull::default(),
            "The Mandalorian is chock-full of references and callbacks, mostly to previous Star Wars content.",
        );
    }

    #[test]
    fn lower_space_choked() {
        assert_suggestion_result(
            "I was making decent money, single and choked full of self-importance.",
            ChockFull::default(),
            "I was making decent money, single and chock-full of self-importance.",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case quirks generated 'CHOCK-Full' instead of 'CHOCK-full'"]
    fn upper_space_choked() {
        assert_suggestion_result(
            "Also being a family computer it’s CHOKED full, lots of unnecessary documents no one will sort",
            ChockFull::default(),
            "Also being a family computer it’s CHOCK-full, lots of unnecessary documents no one will sort",
        );
    }
}
