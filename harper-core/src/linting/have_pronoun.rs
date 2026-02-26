use crate::expr::{AnchorStart, Expr, SequenceExpr};
use crate::{Token, TokenKind};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct HavePronoun {
    expr: Box<dyn Expr>,
}

impl Default for HavePronoun {
    fn default() -> Self {
        let expr = SequenceExpr::with(AnchorStart)
            .t_aco("has")
            .t_ws()
            .then_kind_either(
                TokenKind::is_first_person_singular_pronoun,
                TokenKind::is_plural_pronoun,
            );

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for HavePronoun {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // First real word in the match is always "has".
        let has_tok = toks.iter().find(|t| t.kind.is_word())?;
        let span = has_tok.span;
        let original = span.get_content(src);

        Some(Lint {
            span,
            lint_kind: LintKind::Agreement,
            suggestions: vec![Suggestion::replace_with_match_case(
                "have".chars().collect(),
                original,
            )],
            message: "Use `have` with first-person singular or plural pronouns.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Flags questions that begin with `has` followed by a pronoun that requires `have`, \
         such as `Has we …` or `Has I …`, and suggests the correct auxiliary."
    }
}

#[cfg(test)]
mod tests {
    use super::HavePronoun;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_has_we() {
        assert_suggestion_result(
            "Has we finished the report?",
            HavePronoun::default(),
            "Have we finished the report?",
        );
    }

    #[test]
    fn corrects_has_you() {
        assert_suggestion_result(
            "Has you misunderstood?",
            HavePronoun::default(),
            "Have you misunderstood?",
        );
    }

    #[test]
    fn corrects_has_i() {
        assert_suggestion_result(
            "Has I misunderstood?",
            HavePronoun::default(),
            "Have I misunderstood?",
        );
    }

    #[test]
    fn corrects_has_they() {
        assert_suggestion_result(
            "Has they arrived yet?",
            HavePronoun::default(),
            "Have they arrived yet?",
        );
    }

    #[test]
    fn allows_has_he() {
        assert_lint_count("Has he arrived yet?", HavePronoun::default(), 0);
    }

    #[test]
    fn ignores_non_initial_usage() {
        assert_lint_count(
            "The system has we confused for a moment.",
            HavePronoun::default(),
            0,
        );
    }
}
