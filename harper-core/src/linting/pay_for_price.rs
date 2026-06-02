use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct PayForPrice {
    expr: SequenceExpr,
}

impl Default for PayForPrice {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["pay", "paid", "pays", "paying"])
                .t_ws()
                .t_aco("for")
                .t_ws()
                .then_optional(SequenceExpr::default().then_determiner().t_ws())
                .then(
                    SequenceExpr::word_set(&[
                        "bill", "bills", "check", "checks", "cheque", "cheques", "charge",
                        "charges", "cost", "costs", "fee", "fees", "price", "prices",
                    ])
                    .but_not(SequenceExpr::any_of(vec![
                        Box::new(SequenceExpr::aco("check").t_ws_h().t_set(&["in", "out"])),
                        Box::new(SequenceExpr::aco("price").t_ws_h().t_aco("increase")),
                    ])),
                ),
        }
    }
}

impl ExprLinter for PayForPrice {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));
        let span = matched_tokens[2..4].span()?;
        let lint_kind = LintKind::Usage;
        let suggestions = vec![Suggestion::Remove];
        let message = "You `pay for` things or services, but just `pay prices/fees`, etc with no preposition.".to_string();
        Some(Lint {
            span,
            lint_kind,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects extraneous `for` when used of charges, fees, prices, etc."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::PayForPrice;

    #[test]
    fn pay_for_fees() {
        assert_suggestion_result(
            "Accounts having BNB accounts use this to pay for fees",
            PayForPrice::default(),
            "Accounts having BNB accounts use this to pay fees",
        );
    }

    #[test]
    fn pay_for_a_bill() {
        assert_suggestion_result(
            "being instructed to either pay for a bill, an expense, or a gift",
            PayForPrice::default(),
            "being instructed to either pay a bill, an expense, or a gift",
        );
    }

    #[test]
    fn pay_for_a_check() {
        assert_suggestion_result(
            "Having to pay for a check is 'paying for what you use'.",
            PayForPrice::default(),
            "Having to pay a check is 'paying for what you use'.",
        );
    }

    #[test]
    fn pay_for_a_fee() {
        assert_suggestion_result(
            "then the account needs to pay for a fee of X",
            PayForPrice::default(),
            "then the account needs to pay a fee of X",
        );
    }

    #[test]
    fn pay_for_the_bill() {
        assert_suggestion_result(
            "Cannot pay for the bill and cannot delete the bill",
            PayForPrice::default(),
            "Cannot pay the bill and cannot delete the bill",
        );
    }

    #[test]
    fn pay_for_the_check() {
        assert_suggestion_result(
            "should be able to ascertain if he will pay for the check or not",
            PayForPrice::default(),
            "should be able to ascertain if he will pay the check or not",
        );
    }

    #[test]
    fn paying_for_the_bill() {
        assert_suggestion_result(
            "Maybe some people in his family weren't paying for the bill out of their own account",
            PayForPrice::default(),
            "Maybe some people in his family weren't paying the bill out of their own account",
        );
    }

    #[test]
    fn paying_for_the_check() {
        assert_suggestion_result(
            "how does everyone deal with paying for the check or the bill of any activity",
            PayForPrice::default(),
            "how does everyone deal with paying the check or the bill of any activity",
        );
    }

    #[test]
    fn pays_for_the_bill() {
        assert_suggestion_result(
            "I now understand that when a man pays for the bill, he has clear intentions",
            PayForPrice::default(),
            "I now understand that when a man pays the bill, he has clear intentions",
        );
    }

    // Edge cases / False positives / Not yet handlec

    #[test]
    #[ignore = "Detecting this pattery will require more thought"]
    fn dont_flag_paying_for_fees() {
        assert_no_lints(
            "it will specify how much you need and how much you're paying for fees",
            PayForPrice::default(),
        );
    }

    #[test]
    fn dont_flag_pay_for_rate_limit_increase() {
        // Here 'rate' is part of a compound noun 'rate limit increase'
        assert_no_lints(
            "Can we pay for a rate limit increase?",
            PayForPrice::default(),
        );
    }

    #[test]
    fn dont_flag_pay_for_check_in_or_out() {
        assert_no_lints(
            "Most normally, tenants pay for a check-in and landlords pay for the check-out.",
            PayForPrice::default(),
        )
    }

    #[test]
    fn dont_flag_pay_for_the_price_increase() {
        assert_no_lints(
            "Unless you want to pay for the price increase and have no issue with the owner, then go for signing up for a sourcehut account.",
            PayForPrice::default(),
        );
    }

    #[test]
    fn dont_flag_paying_for_comma_prices() {
        assert_no_lints(
            "If everyone used all of the tokens they thought they were paying for, prices would explode.",
            PayForPrice::default(),
        );
    }
}
