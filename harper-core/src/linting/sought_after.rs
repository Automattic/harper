use crate::expr::{Expr, SequenceExpr};
use crate::{Token, TokenKind};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct SoughtAfter {
    expr: Box<dyn Expr>,
}

impl Default for SoughtAfter {
    fn default() -> Self {
        let pattern = SequenceExpr::any_of(vec![
            Box::new(
                SequenceExpr::default()
                    .then_kind_except(TokenKind::is_adverb, &["always", "maybe", "not", "perhaps"]),
            ),
            Box::new(SequenceExpr::word_set(&[
                "abit", // Typo for "a bit"
                "are",  // may cause false positive, but few found so far.
                "bit",
                // "is" causes many false postivies and disambiguating looks tricky.
                // "maybe" causes many false postivies and disambiguating looks tricky.
                "of", "quiet", // Common typo for "quite".
            ])),
        ])
        .t_ws()
        .t_aco("sort")
        .t_ws_h()
        .t_aco("after");

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for SoughtAfter {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks[2].span;

        Some(Lint {
            span,
            lint_kind: LintKind::Eggcorn,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "sought",
                span.get_content(src),
            )],
            message: "The correct word in this context is `sought`.".to_owned(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Correct `sort after` to `sought after`"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::SoughtAfter;

    #[test]
    fn fix_abit_sort_after() {
        assert_suggestion_result(
            "Blue mountain buffalo no damage abit sort after,$120 ono.",
            SoughtAfter::default(),
            "Blue mountain buffalo no damage abit sought after,$120 ono.",
        );
    }

    #[test]
    fn dont_flag_always_sort_after() {
        assert_lint_count(
            "Always sort after converting your set into list objects",
            SoughtAfter::default(),
            0,
        );
    }

    #[test]
    fn fix_are_sort_after() {
        assert_suggestion_result(
            "Ux engineers are sort after, but it requires experience.",
            SoughtAfter::default(),
            "Ux engineers are sought after, but it requires experience.",
        )
    }

    #[test]
    fn fix_are_sort_after_hyphenated() {
        assert_suggestion_result(
            "optimistic people are sort-after for their life",
            SoughtAfter::default(),
            "optimistic people are sought-after for their life",
        );
    }

    #[test]
    fn fix_bit_sort_after() {
        assert_suggestion_result(
            "It's the early enduro model getting a bit sort after now",
            SoughtAfter::default(),
            "It's the early enduro model getting a bit sought after now",
        );
    }

    #[test]
    fn fix_extremely_sort_after() {
        assert_suggestion_result(
            "3 extremely sort after Pokémon trading cards.",
            SoughtAfter::default(),
            "3 extremely sought after Pokémon trading cards.",
        );
    }

    #[test]
    fn fix_fairly_sort_after() {
        assert_suggestion_result(
            "The ability for editors to add tables to pages is a fairly sort after piece of functionality",
            SoughtAfter::default(),
            "The ability for editors to add tables to pages is a fairly sought after piece of functionality",
        );
    }

    #[test]
    fn fix_highly_sort_after() {
        assert_suggestion_result(
            "Wrestlemania 2K adds three highly sort after features",
            SoughtAfter::default(),
            "Wrestlemania 2K adds three highly sought after features",
        );
    }

    #[test]
    fn fix_hugely_sort_after() {
        assert_suggestion_result(
            "Currently the hugely sort after and most highly prized variety is the electric neon blue Paraiba Tourmaline.",
            SoughtAfter::default(),
            "Currently the hugely sought after and most highly prized variety is the electric neon blue Paraiba Tourmaline.",
        );
    }

    #[test]
    fn fix_incredibly_sort_after() {
        assert_suggestion_result(
            "This is no surprise as it's been an incredibly sort after and top choice amongst outdoorists from all walks of life.",
            SoughtAfter::default(),
            "This is no surprise as it's been an incredibly sought after and top choice amongst outdoorists from all walks of life.",
        );
    }

    #[test]
    #[ignore = "'Is' is a bit more subtle to handle correctly"]
    fn fix_is_sort_after() {
        assert_suggestion_result(
            "White bait is sort after by many fisherman because they  are a delicacy.",
            SoughtAfter::default(),
            "White bait is sought after by many fisherman because they  are a delicacy.",
        );
    }

    #[test]
    fn dont_flag_is_sort_after() {
        assert_lint_count(
            "What I would do is sort after the union or join.",
            SoughtAfter::default(),
            0,
        );
    }

    #[test]
    fn fix_kinda_sort_after() {
        assert_suggestion_result(
            "If so is the US bond still kinda sort after as it's tied to USD (that's how I took OPs post).",
            SoughtAfter::default(),
            "If so is the US bond still kinda sought after as it's tied to USD (that's how I took OPs post).",
        );
    }

    #[test]
    fn dont_flag_maybe_sort_after() {
        assert_lint_count(
            "Or maybe sort after adding the index?",
            SoughtAfter::default(),
            0,
        );
    }

    #[test]
    fn fix_most_sort_after() {
        assert_suggestion_result(
            "This has got to be one of the most sort after solutions.",
            SoughtAfter::default(),
            "This has got to be one of the most sought after solutions.",
        );
    }

    #[test]
    fn fix_mostly_sort_after() {
        assert_suggestion_result(
            "What color and size is mostly sort after In ladies footwear",
            SoughtAfter::default(),
            "What color and size is mostly sought after In ladies footwear",
        );
    }

    #[test]
    fn fix_much_sort_after() {
        assert_suggestion_result(
            "Sending audio files is a much sort after feature in chat apps.",
            SoughtAfter::default(),
            "Sending audio files is a much sought after feature in chat apps.",
        );
    }

    #[test]
    fn dont_flag_not_sort_after() {
        assert_lint_count(
            "My issue is that it does not sort after the startyear",
            SoughtAfter::default(),
            0,
        );
    }

    // This part is occasionally sort after and if they were easily available I reckon you'd sell a few easily enough.
    #[test]
    fn fix_occasionally_sort_after() {
        assert_suggestion_result(
            "This part is occasionally sort after and if they were easily available I reckon you'd sell a few easily enough.",
            SoughtAfter::default(),
            "This part is occasionally sought after and if they were easily available I reckon you'd sell a few easily enough.",
        );
    }

    #[test]
    fn fix_of_sort_after() {
        assert_suggestion_result(
            "A couple of sort after casserole pots .",
            SoughtAfter::default(),
            "A couple of sought after casserole pots .",
        );
    }

    #[test]
    fn fix_often_sort_after() {
        assert_suggestion_result(
            "North American countries (ok, there are only two) often sort after the total amount of medals.",
            SoughtAfter::default(),
            "North American countries (ok, there are only two) often sought after the total amount of medals.",
        );
    }

    #[test]
    #[ignore = "'Perhaps' is a bit more subtle to handle correctly"]
    fn fix_perhaps_sort_after() {
        assert_suggestion_result(
            "Perhaps sort after guitar teachers could do a similar teaching tour to a few major cities",
            SoughtAfter::default(),
            "Perhaps sought after guitar teachers could do a similar teaching tour to a few major cities",
        );
    }

    #[test]
    fn dont_flat_perhaps_sort_after() {
        assert_lint_count(
            "min_Vround: perhaps sort after min_breadth.",
            SoughtAfter::default(),
            0,
        );
    }

    #[test]
    fn flag_pretty_sort_after() {
        assert_suggestion_result(
            "But just like jin and V, he is also pretty sort after",
            SoughtAfter::default(),
            "But just like jin and V, he is also pretty sought after",
        );
    }

    #[test]
    fn fix_quiet_sort_after_sic() {
        assert_suggestion_result(
            "MBA in Christ (deemed university) is quiet sort after in South India",
            SoughtAfter::default(),
            "MBA in Christ (deemed university) is quiet sought after in South India",
        );
    }

    // The university that i studied my MBBS from offers the course as well and it is quite sort after for the above said course.
    #[test]
    fn fix_quite_sort_after() {
        assert_suggestion_result(
            "The university that i studied my MBBS from offers the course as well and it is quite sort after for the above said course.",
            SoughtAfter::default(),
            "The university that i studied my MBBS from offers the course as well and it is quite sought after for the above said course.",
        );
    }

    #[test]
    fn fix_rather_sort_after() {
        assert_suggestion_result(
            "In a bid to satisfy an innate inquisitive hunger for a rather sort after phenomenon that only a few could precisely speak",
            SoughtAfter::default(),
            "In a bid to satisfy an innate inquisitive hunger for a rather sought after phenomenon that only a few could precisely speak",
        );
    }

    #[test]
    fn fix_really_sort_after() {
        assert_suggestion_result(
            "Creators - especially women in their 30s, 40s, 50s and 60s are really sort after.",
            SoughtAfter::default(),
            "Creators - especially women in their 30s, 40s, 50s and 60s are really sought after.",
        );
    }

    #[test]
    fn fix_sometimes_sort_after() {
        assert_suggestion_result(
            "the N15 1.6L gearbox is sometimes sort after for the micra",
            SoughtAfter::default(),
            "the N15 1.6L gearbox is sometimes sought after for the micra",
        );
    }

    #[test]
    fn fix_somewhat_sort_after() {
        assert_suggestion_result(
            "I know tri res boots used to be somewhat sort after, but not sure now!",
            SoughtAfter::default(),
            "I know tri res boots used to be somewhat sought after, but not sure now!",
        );
    }

    #[test]
    fn fix_strongly_sort_after() {
        assert_suggestion_result(
            "This eventually leads to the growth that is so strongly sort after.",
            SoughtAfter::default(),
            "This eventually leads to the growth that is so strongly sought after.",
        );
    }

    #[test]
    fn fix_vastly_sort_after() {
        assert_suggestion_result(
            "Hardie stuff no longer vastly sort after as it was years ago , hasn't been for decades.",
            SoughtAfter::default(),
            "Hardie stuff no longer vastly sought after as it was years ago , hasn't been for decades.",
        );
    }

    #[test]
    fn fix_very_sort_after() {
        assert_suggestion_result(
            "I could imagine, this functionality very sort after.",
            SoughtAfter::default(),
            "I could imagine, this functionality very sought after.",
        );
    }
}
