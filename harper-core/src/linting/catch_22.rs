use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct Catch22 {
    expr: SequenceExpr,
}

impl Default for Catch22 {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["cache", "cash"]).then_any_of(vec![
                Box::new(
                    SequenceExpr::default()
                        .t_ws_h()
                        .then(|t: &Token, s: &[char]| {
                            t.kind.is_number() && t.get_ch(s).eq_ch(&['2', '2'])
                        }),
                ),
                Box::new(
                    SequenceExpr::default()
                        .t_ws()
                        .t_aco("twenty")
                        .t_ws_h()
                        .t_aco("two"),
                ),
            ]),
        }
    }
}

impl ExprLinter for Catch22 {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks[0].span,
            lint_kind: LintKind::Malapropism,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "catch",
                toks[0].span.get_content(src),
            )],
            message: "This idiom uses 'catch' instead of 'cache' or 'cash'.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects mistakenly using similar-sounding words in the idiom `catch 22`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Catch22;

    #[test]
    fn fix_cache_22_space() {
        assert_suggestion_result(
            "So, i'm in a bit of a cache 22 situation.",
            Catch22::default(),
            "So, i'm in a bit of a catch 22 situation.",
        );
    }

    #[test]
    fn fix_cache_22_hyphen() {
        assert_suggestion_result(
            "This leads to a cache-22 situation.",
            Catch22::default(),
            "This leads to a catch-22 situation.",
        );
    }

    #[test]
    fn fix_cash_22() {
        assert_suggestion_result(
            "It's literally a cash 22 it's selectivity but also specific criteria",
            Catch22::default(),
            "It's literally a catch 22 it's selectivity but also specific criteria",
        );
    }

    #[test]
    fn fix_cash_twenty_two_space() {
        assert_suggestion_result(
            "It’s a bit of a cash twenty two situation. You see?",
            Catch22::default(),
            "It’s a bit of a catch twenty two situation. You see?",
        );
    }

    #[test]
    fn fix_cash_twenty_two_hyphen() {
        assert_suggestion_result(
            "So, they're they're in a cash twenty-two but in the middle of all of that, we gotta deal with the presenting problem.",
            Catch22::default(),
            "So, they're they're in a catch twenty-two but in the middle of all of that, we gotta deal with the presenting problem.",
        );
    }

    #[test]
    fn fix_cache_twenty_two_hyphen() {
        assert_suggestion_result(
            "The club is in such a cache twenty-two situation given it past experiences with previous managers.",
            Catch22::default(),
            "The club is in such a catch twenty-two situation given it past experiences with previous managers.",
        );
    }
}
