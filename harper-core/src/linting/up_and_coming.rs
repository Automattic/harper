use crate::expr::{Expr, SequenceExpr};
use crate::Token;

use super::expr_linter::Chunk;
use super::{ExprLinter, Lint, LintKind, Suggestion};

/// Nouns for people and acts that are gaining prominence rather than due soon.
const RISING_NOUNS: &[&str] = &[
    "actor",
    "actors",
    "actress",
    "actresses",
    "artist",
    "artists",
    "athlete",
    "athletes",
    "band",
    "bands",
    "coder",
    "coders",
    "comedian",
    "comedians",
    "musician",
    "musicians",
    "player",
    "players",
    "rapper",
    "rappers",
    "singer",
    "singers",
    "star",
    "stars",
    "talent",
    "talents",
];

pub struct UpAndComing {
    expr: Box<dyn Expr>,
}

impl Default for UpAndComing {
    fn default() -> Self {
        let expr = SequenceExpr::aco("upcoming")
            .t_ws()
            .then_optional(SequenceExpr::aco("young").t_ws())
            .t_set(RISING_NOUNS);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for UpAndComing {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "up-and-coming",
                span.get_content(source),
            )],
            message: "Use `up-and-coming` for someone gaining prominence; `upcoming` means due to happen soon.".to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Flags `upcoming` used to mean `up-and-coming` before people such as artists or stars."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::UpAndComing;

    #[test]
    fn fixes_actor() {
        assert_suggestion_result(
            "Have you ever met an upcoming actor, actress or singer who you knew would go far in their career?",
            UpAndComing::default(),
            "Have you ever met an up-and-coming actor, actress or singer who you knew would go far in their career?",
        );
    }

    #[test]
    fn fixes_artists() {
        assert_suggestion_result(
            "Crescendo is a music discovery platform that promotes upcoming artists.",
            UpAndComing::default(),
            "Crescendo is a music discovery platform that promotes up-and-coming artists.",
        );
    }

    #[test]
    fn fixes_young_players() {
        assert_suggestion_result(
            "the upcoming young players and fierce competition from within the squad led him to seek pastures new",
            UpAndComing::default(),
            "the up-and-coming young players and fierce competition from within the squad led him to seek pastures new",
        );
    }

    #[test]
    fn fixes_all_caps() {
        assert_suggestion_result(
            "which is an UPCOMING STAR in my opinion.",
            UpAndComing::default(),
            "which is an UP-AND-COMING STAR in my opinion.",
        );
    }

    #[test]
    fn fixes_coder() {
        assert_suggestion_result(
            "just an upcoming coder lookin to change the world",
            UpAndComing::default(),
            "just an up-and-coming coder lookin to change the world",
        );
    }

    #[test]
    fn allows_upcoming_events() {
        assert_no_lints(
            "The upcoming release fixes it, see you at the upcoming concert.",
            UpAndComing::default(),
        );
    }

    #[test]
    fn allows_upcoming_band_tour() {
        assert_no_lints(
            "Tickets for the upcoming tour are on sale.",
            UpAndComing::default(),
        );
    }
}
