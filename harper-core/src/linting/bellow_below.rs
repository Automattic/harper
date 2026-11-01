use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr, SpelledNumberExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct BellowBelow {
    expr: SequenceExpr,
}

impl Default for BellowBelow {
    fn default() -> Self {
        Self {
            // `bellow` (to shout) is intransitive, so a determiner (including
            // possessive determiners like `their`, `his`, and `its`) or a number
            // right after it almost always means `below` was intended.
            // Only the exact word is matched: `below` is not inflectable, so an
            // inflected form of `bellow` can never be a misspelling of it.
            expr: SequenceExpr::aco("bellow").t_ws().then_any_of([
                Box::new(|tok: &Token, _: &[char]| tok.kind.is_determiner()) as Box<dyn Expr>,
                Box::new(SequenceExpr::number()),
                Box::new(SpelledNumberExpr),
            ]),
        }
    }
}

impl ExprLinter for BellowBelow {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let verb_span = toks.first()?.span;
        let verb_chars = verb_span.get_content(src);

        Some(Lint {
            span: verb_span,
            lint_kind: LintKind::Malapropism,
            suggestions: vec![Suggestion::replace_with_match_case_str("below", verb_chars)],
            message: "`Bellow` means to shout. You probably mean `below`.".to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Looks for `bellow` wrongly used when `below` is intended."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{bellow_below::BellowBelow, tests::assert_no_lints};

    use super::BellowBelow as Rule;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn correct_bellow_the_table() {
        assert_suggestion_result(
            "Going bellow the table",
            Rule::default(),
            "Going below the table",
        );
    }

    #[test]
    fn correct_bellow_the_surface() {
        assert_suggestion_result(
            "The submarine descended bellow the surface",
            Rule::default(),
            "The submarine descended below the surface",
        );
    }

    #[test]
    fn correct_scroll_bellow_the_fold() {
        assert_suggestion_result(
            "Scroll bellow the fold to continue reading",
            Rule::default(),
            "Scroll below the fold to continue reading",
        );
    }

    #[test]
    fn correct_bellow_their_target() {
        assert_suggestion_result(
            "Profits fell bellow their target this quarter",
            Rule::default(),
            "Profits fell below their target this quarter",
        );
    }

    #[test]
    fn correct_bellow_his_knees() {
        assert_suggestion_result(
            "The water only came up to just bellow his knees",
            Rule::default(),
            "The water only came up to just below his knees",
        );
    }

    #[test]
    fn correct_bellow_its_rim() {
        assert_suggestion_result(
            "Fill the jar to just bellow its rim",
            Rule::default(),
            "Fill the jar to just below its rim",
        );
    }

    #[test]
    fn correct_bellow_the_stairs() {
        assert_suggestion_result(
            "She stored the boxes bellow the stairs",
            Rule::default(),
            "She stored the boxes below the stairs",
        );
    }

    #[test]
    fn correct_sentence_initial_case() {
        assert_suggestion_result(
            "Bellow the winter average, the lake froze solid",
            Rule::default(),
            "Below the winter average, the lake froze solid",
        );
    }

    #[test]
    fn correct_bellow_a_threshold() {
        assert_suggestion_result(
            "Do not let the pressure drop bellow a safe threshold",
            Rule::default(),
            "Do not let the pressure drop below a safe threshold",
        );
    }

    #[test]
    fn correct_bellow_digit() {
        assert_suggestion_result(
            "Readings bellow 5 were discarded as noise",
            Rule::default(),
            "Readings below 5 were discarded as noise",
        );
    }

    #[test]
    fn correct_descended_bellow_digit() {
        assert_suggestion_result(
            "The divers descended bellow 30 meters",
            Rule::default(),
            "The divers descended below 30 meters",
        );
    }

    #[test]
    fn correct_bellow_spelled_number() {
        assert_suggestion_result(
            "Keep the solution bellow five degrees",
            Rule::default(),
            "Keep the solution below five degrees",
        );
    }

    #[test]
    fn allow_verb_at_sentence_end() {
        assert_no_lints("He bellowed.", BellowBelow::default());
    }

    #[test]
    fn allow_verb_with_prepositional_phrase() {
        assert_no_lints("She bellows with rage.", BellowBelow::default());
    }

    #[test]
    fn allow_verb_with_adverb() {
        assert_no_lints(
            "Bulls bellow loudly during mating season.",
            BellowBelow::default(),
        );
    }

    #[test]
    fn allow_verb_with_direct_object() {
        assert_no_lints(
            "The sergeant bellowed orders across the parade ground.",
            BellowBelow::default(),
        );
    }

    #[test]
    fn allow_noun_a_bellow() {
        assert_no_lints("He let out a bellow.", BellowBelow::default());
    }

    #[test]
    fn allow_noun_possessive_bellow() {
        assert_no_lints(
            "The bull's bellow echoed across the valley.",
            BellowBelow::default(),
        );
    }

    #[test]
    fn allow_noun_bellows_instrument() {
        assert_no_lints("The blacksmith worked the bellows.", BellowBelow::default());
    }

    #[test]
    fn allow_inflected_bellowing() {
        assert_no_lints("The wind was bellowing all night.", BellowBelow::default());
    }

    #[test]
    fn allow_correct_below() {
        assert_no_lints("Going below the table", BellowBelow::default());
    }
}
