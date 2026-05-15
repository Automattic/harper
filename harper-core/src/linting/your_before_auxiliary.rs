//! Detects "your is", "your are", "your was", "your were" — forms of "be" that are always errors.
//!
//! Also detects "their is", "their are", "their was", "their were" and suggests "there".

use crate::expr::{Expr, SequenceExpr};
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;
use crate::patterns::InflectionOfBe;
use crate::Token;

/// Flags "your is/are/was/were" — these are ALWAYS errors (forms of "be" can never be nouns).
pub struct YourBeforeAuxiliary {
    expr: SequenceExpr,
}

impl Default for YourBeforeAuxiliary {
    fn default() -> Self {
        // Match "your" followed by a form of "be"
        let expr = SequenceExpr::aco("your")
            .t_ws()
            .then(InflectionOfBe::default());

        Self { expr }
    }
}

impl ExprLinter for YourBeforeAuxiliary {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;
        let orig_chars = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case("you're".chars().collect(), orig_chars),
            ],
            message: "Did you mean `you` instead of `your`?".to_owned(),
            priority: 31,
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects `your` used before a form of `be` (your is, your are, your was, your were) and suggests `you're`."
    }
}

/// Flags "their is/are/was/were" and suggests "there".
pub struct TheirBeforeAuxiliary {
    expr: SequenceExpr,
}

impl Default for TheirBeforeAuxiliary {
    fn default() -> Self {
        // Match "their" followed by a form of "be"
        let expr = SequenceExpr::aco("their")
            .t_ws()
            .then(InflectionOfBe::default());

        Self { expr }
    }
}

impl ExprLinter for TheirBeforeAuxiliary {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;
        let orig_chars = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case("there".chars().collect(), orig_chars),
            ],
            message: "Did you mean `there` instead of `their`?".to_owned(),
            priority: 31,
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects `their` used before a form of `be` (their is, their are, their was, their were) and suggests `there`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::{YourBeforeAuxiliary, TheirBeforeAuxiliary};

    // === YourBeforeAuxiliary tests ===

    #[test]
    fn your_is_flagged() {
        assert_lint_count("Your is a good idea.", YourBeforeAuxiliary::default(), 1);
    }

    #[test]
    fn your_is_suggests_youre() {
        assert_suggestion_result(
            "Your is a good idea.",
            YourBeforeAuxiliary::default(),
            "You're a good idea.",
        );
    }

    #[test]
    fn your_are_flagged() {
        assert_lint_count("Your are not alone.", YourBeforeAuxiliary::default(), 1);
    }

    #[test]
    fn your_are_suggests_youre() {
        assert_suggestion_result(
            "Your are not alone.",
            YourBeforeAuxiliary::default(),
            "You're not alone.",
        );
    }

    #[test]
    fn your_was_flagged() {
        assert_lint_count("Your was a mistake.", YourBeforeAuxiliary::default(), 1);
    }

    #[test]
    fn your_were_flagged() {
        assert_lint_count("Your were amazing.", YourBeforeAuxiliary::default(), 1);
    }

    #[test]
    fn your_being_allowed() {
        // "being" can be a noun (gerund), so we don't flag it
        assert_lint_count("Your being here matters.", YourBeforeAuxiliary::default(), 0);
    }

    #[test]
    fn your_doing_allowed() {
        // "doing" can be a noun, so we don't flag it
        assert_lint_count("Your doing this helps.", YourBeforeAuxiliary::default(), 0);
    }

    #[test]
    fn your_will_allowed() {
        // "will" can be a noun (legal document), so we don't flag it
        assert_lint_count("Your will shall be done.", YourBeforeAuxiliary::default(), 0);
    }

    #[test]
    fn your_can_allowed() {
        // "can" is a noun (container), so we don't flag it
        assert_lint_count("Your can is empty.", YourBeforeAuxiliary::default(), 0);
    }

    #[test]
    fn your_must_allowed() {
        // "must" is not typically a noun, but to be conservative we only flag forms of "be"
        assert_lint_count("Your must be done.", YourBeforeAuxiliary::default(), 0);
    }

    #[test]
    fn youre_not_flagged() {
        assert_lint_count("You're a genius.", YourBeforeAuxiliary::default(), 0);
    }

    #[test]
    fn your_possessive_not_flagged() {
        assert_lint_count("Your book is here.", YourBeforeAuxiliary::default(), 0);
    }

    // === TheirBeforeAuxiliary tests ===

    #[test]
    fn their_is_flagged() {
        assert_lint_count("Their is a problem.", TheirBeforeAuxiliary::default(), 1);
    }

    #[test]
    fn their_is_suggests_there() {
        assert_suggestion_result(
            "Their is a problem.",
            TheirBeforeAuxiliary::default(),
            "There is a problem.",
        );
    }

    #[test]
    fn their_are_flagged() {
        assert_lint_count("Their are many issues.", TheirBeforeAuxiliary::default(), 1);
    }

    #[test]
    fn their_are_suggests_there() {
        assert_suggestion_result(
            "Their are many issues.",
            TheirBeforeAuxiliary::default(),
            "There are many issues.",
        );
    }

    #[test]
    fn their_was_flagged() {
        assert_lint_count("Their was a time.", TheirBeforeAuxiliary::default(), 1);
    }

    #[test]
    fn their_were_flagged() {
        assert_lint_count("Their were warriors.", TheirBeforeAuxiliary::default(), 1);
    }

    #[test]
    fn their_can_not_flagged() {
        // "their can" is not a clear error, conservative approach
        assert_lint_count("Their can of worms.", TheirBeforeAuxiliary::default(), 0);
    }

    #[test]
    fn their_possessive_not_flagged() {
        assert_lint_count("Their house is big.", TheirBeforeAuxiliary::default(), 0);
    }

    #[test]
    fn theres_not_flagged() {
        assert_lint_count("There's a way.", TheirBeforeAuxiliary::default(), 0);
    }
}
