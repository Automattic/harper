use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

/// Corrects the homophone confusion between "tale" (story) and "tail" (appendage)
/// in common phrases like "cautionary tale" and "inspirational tale".
pub struct CautionaryTale {
    expr: Box<dyn Expr>,
}

impl Default for CautionaryTale {
    fn default() -> Self {
        let adjectives = WordSet::new(&["cautionary", "inspirational"]);

        let pattern = SequenceExpr::with(adjectives).t_ws().t_aco("tail");

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for CautionaryTale {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tail_span = toks.last()?.span;
        let tail_text = tail_span.get_content(src);

        Some(Lint {
            span: tail_span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case(
                ['t', 'a', 'l', 'e'].to_vec(),
                tail_text,
            )],
            message: "Did you mean `tale` (story)?".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects confusion between `tale` (story) and `tail` (appendage) in common phrases."
    }
}

#[cfg(test)]
mod tests {
    use super::CautionaryTale;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn catches_cautionary_tail() {
        assert_suggestion_result(
            "It serves as a cautionary tail.",
            CautionaryTale::default(),
            "It serves as a cautionary tale.",
        );
    }

    #[test]
    fn catches_inspirational_tail() {
        assert_suggestion_result(
            "Her journey is an inspirational tail of perseverance.",
            CautionaryTale::default(),
            "Her journey is an inspirational tale of perseverance.",
        );
    }

    #[test]
    fn catches_capitalized_cautionary_tail() {
        assert_suggestion_result(
            "The article discusses a Cautionary Tail about privacy.",
            CautionaryTale::default(),
            "The article discusses a Cautionary Tale about privacy.",
        );
    }

    #[test]
    fn catches_uppercase_cautionary_tail() {
        assert_suggestion_result(
            "THE STORY IS A CAUTIONARY TAIL.",
            CautionaryTale::default(),
            "THE STORY IS A CAUTIONARY TALE.",
        );
    }

    #[test]
    fn catches_mixed_case() {
        assert_suggestion_result(
            "This serves as an inspirational Tail for all.",
            CautionaryTale::default(),
            "This serves as an inspirational Tale for all.",
        );
    }

    #[test]
    fn allows_actual_tail() {
        assert_lint_count(
            "The dog wagged its tail happily.",
            CautionaryTale::default(),
            0,
        );
    }

    #[test]
    fn allows_different_adjective_with_tail() {
        assert_lint_count("The cat has a long tail.", CautionaryTale::default(), 0);
    }

    #[test]
    fn allows_correct_tale() {
        assert_lint_count(
            "It serves as a cautionary tale.",
            CautionaryTale::default(),
            0,
        );
    }

    #[test]
    fn allows_inspirational_tale() {
        assert_lint_count(
            "Her story is an inspirational tale.",
            CautionaryTale::default(),
            0,
        );
    }

    #[test]
    fn catches_in_longer_text() {
        assert_suggestion_result(
            "The movie presents a cautionary tail about the dangers of AI. It's really scary.",
            CautionaryTale::default(),
            "The movie presents a cautionary tale about the dangers of AI. It's really scary.",
        );
    }

    #[test]
    fn catches_multiple_occurrences() {
        assert_lint_count(
            "This cautionary tail is also an inspirational tail about overcoming adversity.",
            CautionaryTale::default(),
            2,
        );
    }

    #[test]
    fn allows_tail_in_different_context() {
        assert_lint_count(
            "The inspirational speaker told the tale of a dog's tail.",
            CautionaryTale::default(),
            0,
        );
    }

    #[test]
    fn catches_at_start_of_sentence() {
        assert_suggestion_result(
            "Cautionary tail: don't trust strangers.",
            CautionaryTale::default(),
            "Cautionary tale: don't trust strangers.",
        );
    }
}
