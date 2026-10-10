use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct NobelNoble {
    expr: FirstMatchOf,
}

impl Default for NobelNoble {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::word_set(["nobel", "noble"])
                        .t_ws()
                        .t_set(["peace", "piece"])
                        .t_ws()
                        .t_set(["price", "prize", "prise", "prices", "prizes", "prises"])
                        .but_not(
                            SequenceExpr::word_seq(&["nobel", "peace"])
                                .t_ws()
                                .t_set(["prize", "prizes"]),
                        ),
                ) as Box<dyn Expr>,
                Box::new(SequenceExpr::aco("noble").t_ws().t_set([
                    "foundation",
                    "laureate",
                    "laureates",
                    "prize",
                    "prizes",
                ])),
            ]),
        }
    }
}

impl ExprLinter for NobelNoble {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.span()?;

        let lint_kind = LintKind::Spelling;
        let message =
            "Corrects `noble` to `Nobel` when referring to the Nobel Peace Prize".to_owned();

        match toks.len() {
            5 => {
                let replacement = if span
                    .get_content(src)
                    .ends_with_ignore_ascii_case_chars(&['s'])
                {
                    "Nobel Peace Prizes"
                } else {
                    "Nobel Peace Prize"
                };
                Some(Lint {
                    span,
                    lint_kind,
                    suggestions: vec![Suggestion::replace_with_match_case_str(
                        replacement,
                        span.get_content(src),
                    )],
                    message,
                    ..Default::default()
                })
            }
            3 => Some(Lint {
                span: toks[0].span,
                lint_kind,
                suggestions: vec![Suggestion::replace_with_match_case_str(
                    "Nobel",
                    toks[0].span.get_content(src),
                )],
                message,
                ..Default::default()
            }),
            _ => None,
        }
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects misspellings of `Nobel` Peace Prize."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::NobelNoble;

    // From the Weir rule

    #[test]
    fn noble_peace_price() {
        assert_suggestion_result(
            "He accepted the Noble Peace Price at the ceremony.",
            NobelNoble::default(),
            "He accepted the Nobel Peace Prize at the ceremony.",
        );
    }

    #[test]
    #[ignore = "'replace_with_match_case' issue"]
    fn noble_peace_price_timeline() {
        assert_suggestion_result(
            "Our timeline mentions the Noble peace price until 1950.",
            NobelNoble::default(),
            "Our timeline mentions the Nobel Peace Prize until 1950.",
        );
    }

    #[test]
    #[ignore = "'replace_with_match_case' issue"]
    fn noble_peace_price_facts() {
        assert_suggestion_result(
            "noble peace price facts are easy to recap.",
            NobelNoble::default(),
            "Nobel Peace Prize facts are easy to recap.",
        );
    }

    #[test]
    #[ignore = "'replace_with_match_case' issue"]
    fn noble_piece_price_committees() {
        assert_suggestion_result(
            "Nobel piece price committees meet tonight.",
            NobelNoble::default(),
            "Nobel Peace Prize committees meet tonight.",
        );
    }

    #[test]
    #[ignore = "'replace_with_match_case' issue"]
    fn noble_piece_prize_winners() {
        assert_suggestion_result(
            "nobel piece prize winners shared stories.",
            NobelNoble::default(),
            "Nobel Peace Prize winners shared stories.",
        );
    }

    #[test]
    #[ignore = "'replace_with_match_case' issue"]
    fn noble_piece_prise_ceremonies() {
        assert_suggestion_result(
            "Nobel piece prise ceremonies look ready.",
            NobelNoble::default(),
            "Nobel Peace Prize ceremonies look ready.",
        );
    }

    #[test]
    #[ignore = "'replace_with_match_case' issue"]
    fn noble_piece_price_stands_out() {
        assert_suggestion_result(
            "NOBLE PIECE PRICE stands out in the article.",
            NobelNoble::default(),
            "Nobel Peace Prize stands out in the article.",
        );
    }

    #[test]
    #[ignore = "'replace_with_match_case' issue"]
    fn noble_piece_price_memo() {
        assert_suggestion_result(
            "She referenced the noble piece price memo.",
            NobelNoble::default(),
            "She referenced the Nobel Peace Prize memo.",
        );
    }

    #[test]
    fn noble_peace_prise_references() {
        assert_suggestion_result(
            "Noble Peace Prise references appear frequently.",
            NobelNoble::default(),
            "Nobel Peace Prize references appear frequently.",
        );
    }

    #[test]
    #[ignore = "'replace_with_match_case' issue"]
    fn noble_peace_prise_news() {
        assert_suggestion_result(
            "nobel peace prise news alerted the team.",
            NobelNoble::default(),
            "Nobel Peace Prize news alerted the team.",
        );
    }

    #[test]
    #[ignore = "'replace_with_match_case' issue"]
    fn noble_piece_price_reports() {
        assert_suggestion_result(
            "nobel piece price reports track the conference.",
            NobelNoble::default(),
            "Nobel Peace Prize reports track the conference.",
        );
    }

    #[test]
    #[ignore = "'replace_with_match_case' issue"]
    fn noble_piece_prise_historian() {
        assert_suggestion_result(
            "The noble piece prise historian wrote a book.",
            NobelNoble::default(),
            "The Nobel Peace Prize historian wrote a book.",
        );
    }

    // True negatives / false positives

    #[test]
    fn nobel_peace_prize_ceremony() {
        assert_no_lints(
            "The Nobel Peace Prize ceremony is broadcast worldwide.",
            NobelNoble::default(),
        );
    }

    #[test]
    fn nobel_prize_for_peace() {
        assert_no_lints(
            "Nobel Prize for Peace winners remain inspiring.",
            NobelNoble::default(),
        );
    }

    #[test]
    fn peace_prize_winners() {
        assert_no_lints(
            "Peace Prize winners celebrate after the award.",
            NobelNoble::default(),
        );
    }

    // Known false negatives

    // #[test]
    // #[ignore = "This seems like an unlikely sentence to find in the real world"]
    // fn nobel_peace_prices() {
    //     assert_suggestion_result(
    //         "The Nobel Peace Prices for the exhibition confuse some visitors.",
    //         NobelNoble::default(),
    //         "The Nobel Peace Prizes for the exhibition confuse some visitors.",
    //     );
    // }

    #[test]
    // #[ignore]
    fn noble_peace_prizes_across_years() {
        assert_suggestion_result(
            "Stories mention the Noble Peace Prizes across years.",
            NobelNoble::default(),
            "Stories mention the Nobel Peace Prizes across years.",
        );
    }

    // New ones from GitHub via Google

    #[test]
    fn noble_foundation() {
        assert_suggestion_result(
            "The data used in this project is taken from The Noble Foundation which contains data about the Noble Prize from 1901 to 2016.",
            NobelNoble::default(),
            "The data used in this project is taken from The Nobel Foundation which contains data about the Nobel Prize from 1901 to 2016.",
        );
    }
}
