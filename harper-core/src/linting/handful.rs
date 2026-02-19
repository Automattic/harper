use crate::expr::{Expr, SequenceExpr, SpaceOrHyphen};
use crate::linting::expr_linter::Chunk;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct Handful {
    expr: Box<dyn Expr>,
}

impl Default for Handful {
    fn default() -> Self {
        let expr = SequenceExpr::any_capitalization_of("hand")
            .then_one_or_more(SpaceOrHyphen)
            .then_any_capitalization_of("full")
            .then_one_or_more(SpaceOrHyphen)
            .then_any_capitalization_of("of");

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for Handful {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        if matched_tokens.len() < 2 {
            return None;
        }

        let mut highlight_end = matched_tokens.len() - 1;
        while highlight_end > 0 {
            let prev = &matched_tokens[highlight_end - 1];
            if prev.kind.is_whitespace() || prev.kind.is_hyphen() {
                highlight_end -= 1;
            } else {
                break;
            }
        }

        if highlight_end == 0 {
            return None;
        }

        let replacement = &matched_tokens[..highlight_end];
        let span = replacement.span()?;
        let template = matched_tokens.first()?.span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::BoundaryError,
            suggestions: vec![Suggestion::replace_with_match_case(
                "handful".chars().collect(),
                template,
            )],
            message: "Write this quantity as the single word `handful`.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Keeps the palm-sized quantity expressed by `handful` as one word."
    }
}

#[cfg(test)]
mod tests {
    use super::Handful;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn suggests_plain_spacing() {
        assert_suggestion_result(
            "Her basket held a hand full of berries.",
            Handful::default(),
            "Her basket held a handful of berries.",
        );
    }

    #[test]
    fn suggests_capitalized_form() {
        assert_suggestion_result(
            "Hand full of tales lined the shelf.",
            Handful::default(),
            "Handful of tales lined the shelf.",
        );
    }

    #[test]
    fn suggests_hyphenated_form() {
        assert_suggestion_result(
            "A hand-full of marbles scattered across the floor.",
            Handful::default(),
            "A handful of marbles scattered across the floor.",
        );
    }

    #[test]
    fn suggests_space_hyphen_combo() {
        assert_suggestion_result(
            "A hand - full of seeds spilled on the workbench.",
            Handful::default(),
            "A handful of seeds spilled on the workbench.",
        );
    }

    #[test]
    fn suggests_initial_hyphen_variants() {
        assert_suggestion_result(
            "Hand-Full of furniture, the cart creaked slowly.",
            Handful::default(),
            "Handful of furniture, the cart creaked slowly.",
        );
    }

    #[test]
    fn flags_multiple_instances() {
        assert_lint_count(
            "She carried a hand full of carrots and a hand full of radishes.",
            Handful::default(),
            2,
        );
    }

    #[test]
    fn allows_correct_handful() {
        assert_no_lints(
            "A handful of volunteers arrived in time.",
            Handful::default(),
        );
    }

    #[test]
    fn allows_parenthetical_hand() {
        assert_no_lints(
            "His hand, full of ink, kept writing without pause.",
            Handful::default(),
        );
    }

    #[test]
    fn allows_hand_is_full() {
        assert_no_lints("The hand is full of water.", Handful::default());
    }

    #[test]
    fn allows_handfull_typo() {
        assert_no_lints(
            "The word handfull is an incorrect spelling.",
            Handful::default(),
        );
    }
}
