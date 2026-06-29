use crate::{
    Lint, Span, Token,
    char_string::CharStringExt,
    expr::{AnchorEnd, Expr, SequenceExpr, SpelledNumberExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

pub struct FallBelow {
    expr: SequenceExpr,
}

impl Default for FallBelow {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["fall", "fallen", "falling", "falls", "fell"])
                .t_ws()
                .then_any_of([
                    Box::new(SequenceExpr::number()) as Box<dyn Expr>,
                    Box::new(SpelledNumberExpr),
                ])
                .t_ws()
                .t_set(&["feet", "meters", "metres"])
                .t_ws()
                .t_aco("below")
                .then_any_of(vec![
                    Box::new(AnchorEnd) as Box<dyn Expr>,
                    Box::new(SequenceExpr::whitespace().t_aco("and")),
                ]),
        }
    }
}

impl ExprLinter for FallBelow {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let below_span = find_the_only_token_matching(matched_tokens, source, |t, s| {
            t.get_ch(s).eq_str("below")
        })?
        .span;

        // extend span backwards to include the whitespace before "below"
        let new_span = Span::new(below_span.start.saturating_sub(1), below_span.end);

        Some(Lint {
            span: new_span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::Remove],
            message: "The word below is unnecessary here. Consider dropping it or specifying what the subject fell below.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags redundant usage of `below` after fall distances."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::FallBelow;

    #[test]
    fn fix_40_feet() {
        assert_suggestion_result(
            "However, there wasn't, causing Gurman to fall 40 feet below.",
            FallBelow::default(),
            "However, there wasn't, causing Gurman to fall 40 feet.",
        );
    }

    #[test]
    fn fix_130_feet() {
        assert_suggestion_result(
            "Woman who died in Brazil Bungee jump tragedy was alive after falling 130 feet below",
            FallBelow::default(),
            "Woman who died in Brazil Bungee jump tragedy was alive after falling 130 feet",
        );
    }

    #[test]
    fn fix_300_feet() {
        assert_suggestion_result(
            "The young trekker was stuck in Bramhagiri Rocks in Nandi Hills after slipping and falling 300 feet below, said PRO Defence.",
            FallBelow::default(),
            "The young trekker was stuck in Bramhagiri Rocks in Nandi Hills after slipping and falling 300 feet, said PRO Defence.",
        );
    }

    #[test]
    fn fix_fell_20_feet() {
        assert_suggestion_result(
            "But while jumping, unfortunately, the rope broke and he fell 20 feet below.",
            FallBelow::default(),
            "But while jumping, unfortunately, the rope broke and he fell 20 feet.",
        );
    }

    #[test]
    fn fix_30_feet() {
        assert_suggestion_result(
            "“Mike and Annie call for the lift to stop, but the chair did not stop. Instead it carried her up, and Annie fell 30 feet below, landing on her skis,” Miller’s attorney, Bruce Braley, said.",
            FallBelow::default(),
            "“Mike and Annie call for the lift to stop, but the chair did not stop. Instead it carried her up, and Annie fell 30 feet, landing on her skis,” Miller’s attorney, Bruce Braley, said.",
        );
    }

    #[test]
    fn fix_50_feet() {
        assert_suggestion_result(
            "SUV crashes through bridge guardrail, falls 50 feet below.",
            FallBelow::default(),
            "SUV crashes through bridge guardrail, falls 50 feet.",
        );
    }

    #[test]
    fn fix_falls_20_feet() {
        assert_suggestion_result(
            "Motorcyclist crashes off edge of Ballard Bridge, falls 20 feet below",
            FallBelow::default(),
            "Motorcyclist crashes off edge of Ballard Bridge, falls 20 feet",
        );
    }

    #[test]
    fn fix_fell_20_meters_and() {
        assert_suggestion_result(
            "On the 18th, a man fell 20 meters below and died in Songnisan National Park.",
            FallBelow::default(),
            "On the 18th, a man fell 20 meters and died in Songnisan National Park.",
        );
    }

    // Known failures

    #[test]
    #[ignore = "We can't detect when the chunk does not end after 'below'"]
    fn cant_fix_several_metres_below() {
        assert_suggestion_result(
            "A large billboard on Krystal Point building fell several metres below damaging several vehicles.",
            FallBelow::default(),
            "A large billboard on Krystal Point building fell several metres damaging several vehicles.",
        );
    }

    #[test]
    #[ignore = "We can't detect when the chunk does not end after 'below'"]
    fn cant_fix_fall_40_feet() {
        assert_suggestion_result(
            "Speeding Bike Collides with Railing, 3 Riders Fall 40 Feet Below in Visakhapatnam.",
            FallBelow::default(),
            "Speeding Bike Collides with Railing, 3 Riders Fall 40 Feet in Visakhapatnam.",
        );
    }
}
