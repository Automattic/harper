use crate::TokenKind;
use crate::expr::{AnchorStart, Expr, ExprMap, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

/// Detects the eggcorn `out to be` when the intended phrase is `ought to be`.
///
/// Legitimate phrasal-verb uses like `turn out to be`, `work out to be`, or
/// `make it out to be` are ignored.
pub struct OughtToBe {
    expr: Box<dyn Expr>,
    map: std::sync::Arc<ExprMap<usize>>, // index of the `out` token within the match
}

impl Default for OughtToBe {
    fn default() -> Self {
        use std::sync::Arc;

        // We’ll construct three branches and record where the `out` token sits in each.
        // 1) non-verb word + pronoun + "out to be"  → index of `out` = 4 tokens after start
        //    [word(!verb)] [ws] [pronoun] [ws] [out] [ws] [to] [ws] [be]
        let branch_nonverb_pronoun = SequenceExpr::default()
            .then_kind_is_but_is_not(TokenKind::is_word, TokenKind::is_verb)
            .then_whitespace()
            .then_pronoun()
            .then_whitespace()
            .then_fixed_phrase("out to be");

        // 2) start-of-sentence + pronoun + "out to be" → index of `out` = 2 tokens after start
        //    [AnchorStart] [pronoun] [ws] [out] [ws] [to] [ws] [be]
        let branch_anchor_pronoun = SequenceExpr::with(AnchorStart)
            .then_pronoun()
            .then_whitespace()
            .then_fixed_phrase("out to be");

        // 3) punctuation + pronoun + "out to be" → index of `out` = 4 tokens after start
        //    [punct] [ws] [pronoun] [ws] [out] [ws] [to] [ws] [be]
        let branch_punct_pronoun = SequenceExpr::default()
            .then_punctuation()
            .then_whitespace()
            .then_pronoun()
            .then_whitespace()
            .then_fixed_phrase("out to be");

        let mut map = ExprMap::default();
        map.insert(branch_nonverb_pronoun, 4);
        map.insert(branch_anchor_pronoun, 2);
        map.insert(branch_punct_pronoun, 4);

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for OughtToBe {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched: &[Token], source: &[char]) -> Option<Lint> {
        // Find which branch matched and where the `out` token sits.
        let out_index = *self.map.lookup(0, matched, source)?;
        let out_tok = matched.get(out_index)?;
        let replace_span = out_tok.span; // only replace the word `out`
        let original = replace_span.get_content(source);
        Some(Lint {
            span: replace_span,
            lint_kind: LintKind::Eggcorn,
            suggestions: vec![Suggestion::replace_with_match_case(
                "ought".chars().collect(),
                original,
            )],
            message: "Did you mean `ought to be` (expressing expectation or obligation)?"
                .to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Detects the mistaken `out to be` and suggests `ought to be`, while ignoring legitimate phrasal-verb uses such as `turn out to be` and `make it out to be`."
    }
}

#[cfg(test)]
mod tests {
    use super::OughtToBe;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    // Flagged examples
    #[test]
    fn flags_you_out_to_be_able_to_see() {
        assert_suggestion_result(
            "you out to be able to see",
            OughtToBe::default(),
            "you ought to be able to see",
        );
    }

    #[test]
    fn flags_as_it_out_to_be() {
        assert_suggestion_result("as it out to be", OughtToBe::default(), "as it ought to be");
    }

    #[test]
    fn flags_then_it_out_to_be() {
        assert_suggestion_result(
            "then it out to be",
            OughtToBe::default(),
            "then it ought to be",
        );
    }

    // Legit phrasal-verb cases that should be ignored
    #[test]
    fn ignores_turned_out_to_be() {
        assert_lint_count("It turned out to be fine.", OughtToBe::default(), 0);
    }

    #[test]
    fn ignores_turns_out_to_be() {
        assert_lint_count("It turns out to be fine.", OughtToBe::default(), 0);
    }

    #[test]
    fn ignores_make_it_out_to_be() {
        assert_lint_count(
            "It's not as simple as they make it out to be.",
            OughtToBe::default(),
            0,
        );
    }

    #[test]
    fn ignores_makes_it_out_to_be() {
        assert_lint_count(
            "I think this rule may not be as smart as its definition makes it out to be.",
            OughtToBe::default(),
            0,
        );
    }

    #[test]
    fn ignores_worked_out_to_be() {
        assert_lint_count("It worked out to be $5.", OughtToBe::default(), 0);
    }

    #[test]
    fn ignores_figured_it_out_to_be() {
        assert_lint_count(
            "I figured it out to be a memory issue.",
            OughtToBe::default(),
            0,
        );
    }

    #[test]
    fn ignores_try_it_out_to_be() {
        assert_lint_count("Try it out to be sure.", OughtToBe::default(), 0);
    }

    #[test]
    fn ignores_separate_it_out_to_be() {
        assert_lint_count(
            "I want to separate it out to be able to process it later.",
            OughtToBe::default(),
            0,
        );
    }

    #[test]
    fn ignores_rotate_it_out_to_be() {
        assert_lint_count(
            "We will rotate it out to be eventually deleted.",
            OughtToBe::default(),
            0,
        );
    }

    #[test]
    fn ignores_flesh_it_out_to_be() {
        assert_lint_count(
            "This needs some work to flesh it out to be usable.",
            OughtToBe::default(),
            0,
        );
    }
}
