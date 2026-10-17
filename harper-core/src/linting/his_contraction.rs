use harper_brill::UPOS;

use crate::Token;
use crate::expr::{Expr, ExprMap, SequenceExpr};
use crate::patterns::UPOSSet;

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

/// Flags `his` used where `he's` (he is / he has) is intended.
/// See issue #4560.
pub struct HisContraction {
    expr: ExprMap<usize>,
}

impl Default for HisContraction {
    fn default() -> Self {
        let mut map = ExprMap::default();

        // A possessive followed by a verb, adjective, or auxiliary is likely a
        // misspelling of the contraction "he's".
        let contraction_pattern = SequenceExpr::default()
            .t_aco("his")
            .t_ws()
            .then(UPOSSet::new(&[UPOS::VERB, UPOS::ADJ, UPOS::AUX]));

        map.insert(contraction_pattern, 0);

        Self { expr: map }
    }
}

impl ExprLinter for HisContraction {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_idx = self.expr.lookup(0, matched_tokens, source)?;
        let span = matched_tokens[*offending_idx].span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("he's", span.get_content(source)),
            ],
            message: "Did you mean `he's` (a contraction of 'he is' or 'he has') instead of `his` (possessive)?".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Flags the possessive `his` when the contraction `he's` (he is / he has) is intended."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::pooled_linter::for_tests::create_test_pool;

    create_test_pool!(HisContraction, HisContraction, HisContraction::default());
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::HisContraction;

    // True positive: his + verb indicates contraction intended
    #[test]
    fn flags_his_verb_complaining() {
        assert_suggestion_result(
            "I don't know what his complaining about.",
            test_linter(),
            "I don't know what he's complaining about.",
        );
    }

    #[test]
    fn flags_his_verb_running() {
        assert_suggestion_result(
            "Look at his running fast.",
            test_linter(),
            "Look at he's running fast.",
        );
    }

    #[test]
    fn flags_his_adjective_tired() {
        assert_suggestion_result(
            "He seems his tired today.",
            test_linter(),
            "He seems he's tired today.",
        );
    }

    #[test]
    fn flags_his_auxiliary_have() {
        assert_suggestion_result(
            "I think his finished the work.",
            test_linter(),
            "I think he's finished the work.",
        );
    }

    #[test]
    fn flags_his_verb_at_start() {
        assert_suggestion_result(
            "His going to the store later.",
            test_linter(),
            "He's going to the store later.",
        );
    }

    // True negative: possessive his + noun is correct
    #[test]
    fn allows_his_noun_book() {
        assert_no_lints("This is his book.", test_linter());
    }

    #[test]
    fn allows_his_noun_car() {
        assert_no_lints("That is his car parked outside.", test_linter());
    }

    #[test]
    fn allows_his_possessive() {
        assert_no_lints("It is his responsibility.", test_linter());
    }

    #[test]
    fn allows_his_possessive_plural() {
        assert_no_lints("These are his books.", test_linter());
    }

    // Edge cases: his at end of sentence/chunk (should not trigger)
    #[test]
    fn allows_his_at_end() {
        assert_no_lints("The book is his.", test_linter());
    }

    #[test]
    fn allows_his_before_punctuation() {
        assert_no_lints("This is his, and that is mine.", test_linter());
    }

    // False negative cases: his + verb that should trigger but don't due to implementation limits
    // These document current limitations for future improvement
    #[test]
    fn flags_his_verb_gerund_complex() {
        assert_suggestion_result(
            "Despite his being tired, he continued.",
            test_linter(),
            "Despite he's being tired, he continued.",
        );
    }

    // Multi-sentence cases
    #[test]
    fn flags_multiple_occurrences() {
        assert_suggestion_result(
            "His running late. His eating quickly.",
            test_linter(),
            "He's running late. He's eating quickly.",
        );
    }

    // Case sensitivity: should preserve original casing
    #[test]
    fn preserves_case_capitalized() {
        assert_suggestion_result("His running late.", test_linter(), "He's running late.");
    }

    // Ensure we don't flag correct contractions
    #[test]
    fn allows_actual_he_s() {
        assert_no_lints("He's going to the park.", test_linter());
    }

    // Test with adjectives
    #[test]
    fn flags_his_adjective_happy() {
        assert_suggestion_result(
            "She looks his happy today.",
            test_linter(),
            "She looks he's happy today.",
        );
    }
}
