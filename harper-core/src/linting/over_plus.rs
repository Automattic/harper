use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{All, Expr, OwnedExprExt, SequenceExpr, SpelledNumberExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_hyphen, followed_by_word},
    },
    patterns::Word,
};

pub struct OverPlus {
    expr: All,
}

impl Default for OverPlus {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("over")
                .t_ws()
                .then_any_of(vec![
                    Box::new(SpelledNumberExpr),
                    Box::new(|tok: &Token, _src: &[char]| tok.kind.is_number()),
                ])
                .then_optional(
                    SequenceExpr::whitespace().t_set(&["thousand", "million", "billion"]),
                )
                .t_ws()
                .then(SequenceExpr::aco("plus").but_not(Word::new_exact("PLUS")))
                .but_not(SequenceExpr::anything().t_any().t_aco("one")),
        }
    }
}

impl ExprLinter for OverPlus {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if followed_by_hyphen(ctx)
            || followed_by_word(ctx, |t| {
                t.get_ch(src)
                    .eq_any_ignore_ascii_case_str(&["innings", "size"])
            })
        {
            return None;
        }

        let slices = [toks.get_rel_slice(0, -3)?, &toks[2..]];

        let span = toks.span()?;

        let suggestions = slices
            .iter()
            .map(|t| {
                Suggestion::replace_with_match_case(
                    t.get_ch(src).unwrap().to_vec(),
                    span.get_content(src),
                )
            })
            .collect();

        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions,
            message: "It's redundant to use both `over` and `plus`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detected redundant use of `over` and `plus` used together to bracket a number."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_no_lints};

    use super::OverPlus;

    #[test]
    fn test_skeleton() {
        assert_good_and_bad_suggestions(
            "We looked through over 200 plus photos",
            OverPlus::default(),
            &[
                "We looked through over 200 photos",
                "We looked through 200 plus photos",
            ],
            &[],
        );
    }

    #[test]
    fn test_100_million() {
        assert_good_and_bad_suggestions(
            "$ 19 Billion dollars so far, with over 100 million plus views!",
            OverPlus::default(),
            &[
                "$ 19 Billion dollars so far, with over 100 million views!",
                "$ 19 Billion dollars so far, with 100 million plus views!",
            ],
            &[],
        );
    }

    #[test]
    fn test_5000_plus() {
        assert_good_and_bad_suggestions(
            "Over 5000 plus participants of the Global Conference",
            OverPlus::default(),
            &[
                "Over 5000 participants of the Global Conference",
                // TODO: Due to quirks of `replace_with_match_case()` and `copy_casing()`,
                // TODO: "plus" gets unexpectedly uppercased to "Plus"
                // "5000 plus participants of the Global Conference",
            ],
            &[],
        );
    }

    #[test]
    fn test_18() {
        assert_good_and_bad_suggestions(
            "over 18 plus only censored",
            OverPlus::default(),
            &["over 18 only censored", "18 plus only censored"],
            &[],
        );
    }

    #[test]
    fn test_130() {
        assert_good_and_bad_suggestions(
            "100% Halal certified international brand with over 130 plus locations",
            OverPlus::default(),
            &[
                "100% Halal certified international brand with over 130 locations",
                "100% Halal certified international brand with 130 plus locations",
            ],
            &[],
        );
    }

    #[test]
    fn dont_flag_plus_hyphen() {
        assert_no_lints(
            "Currently, there are over one million Plus-linked ATMs in 170 countries worldwide",
            OverPlus::default(),
        );
    }

    #[test]
    fn dont_flag_plus_size() {
        assert_no_lints("Over 50 plus size outfits", OverPlus::default());
    }

    #[test]
    fn dont_flag_hitless_innings() {
        assert_no_lints(
            "like the Cardinals, who went hitless over five-plus innings",
            OverPlus::default(),
        );
    }

    #[test]
    fn dont_flag_one_plus_phone() {
        assert_no_lints(
            "OnePlus 9 Series Wifi keep disconnecting over one plus 9 Every time",
            OverPlus::default(),
        );
    }

    #[test]
    fn over_160_plus() {
        assert_good_and_bad_suggestions(
            "reached approximately 25,00,000 Indian learners with over 160 plus unique online courses",
            OverPlus::default(),
            &[
                "reached approximately 25,00,000 Indian learners with over 160 unique online courses",
                "reached approximately 25,00,000 Indian learners with 160 plus unique online courses",
            ],
            &[],
        );
    }

    #[test]
    fn over_20_plus_years() {
        assert_good_and_bad_suggestions(
            "Marketing professional with over 20 plus years of domestic and international marketing experience",
            OverPlus::default(),
            &[
                "Marketing professional with over 20 years of domestic and international marketing experience",
                "Marketing professional with 20 plus years of domestic and international marketing experience",
            ],
            &[],
        );
    }

    #[test]
    fn dont_flag_scoreless_innings() {
        assert_no_lints(
            "Jacob Latz K's four over six-plus scoreless innings",
            OverPlus::default(),
        );
    }

    #[test]
    fn dont_flag_plus_all_caps() {
        assert_no_lints(
            "over 5,000 PLUS personnel will be on duty to assist highway users",
            OverPlus::default(),
        );
    }
}
