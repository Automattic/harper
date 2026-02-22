use crate::expr::Expr;
use crate::expr::FirstMatchOf;
use crate::expr::SequenceExpr;
use crate::{Token, TokenStringExt, patterns::WordSet};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct AmountsFor {
    expr: Box<dyn Expr>,
}

impl Default for AmountsFor {
    fn default() -> Self {
        let singular_context = WordSet::new(&["that", "which", "it", "this"]);

        let singular_pattern = SequenceExpr::with(singular_context)
            .then_whitespace()
            .then_fixed_phrase("amounts for");

        let singular_context = WordSet::new(&[
            "they", "can", "could", "may", "might", "must", "should", "will", "would",
        ]);

        let plural_pattern = SequenceExpr::with(singular_context)
            .then_whitespace()
            .then_fixed_phrase("amount for");

        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                Box::new(singular_pattern),
                Box::new(plural_pattern),
            ])),
        }
    }
}

impl ExprLinter for AmountsFor {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let content = toks.span()?.get_content_string(src).to_lowercase();

        if content.ends_with("amounts for") {
            let span = toks[2..5].span()?;

            return Some(Lint {
                span,
                lint_kind: LintKind::WordChoice,
                suggestions: vec![
                    Suggestion::replace_with_match_case(
                        "amounts to".chars().collect(),
                        span.get_content(src),
                    ),
                    Suggestion::replace_with_match_case(
                        "accounts for".chars().collect(),
                        span.get_content(src),
                    ),
                ],
                message: "`amounts for` is not idiomatic English. You probably meant `amounts to` or `accounts for`.".to_owned(),
                priority: 63,
            });
        }

        if content.ends_with("amount for") {
            let span = toks[2..5].span()?;

            return Some(Lint {
                span,
                lint_kind: LintKind::WordChoice,
                suggestions: vec![
                    Suggestion::replace_with_match_case(
                        "amount to".chars().collect(),
                        span.get_content(src),
                    ),
                    Suggestion::replace_with_match_case(
                        "account for".chars().collect(),
                        span.get_content(src),
                    ),
                ],
                message: "`amounts for` is not idiomatic English. You probably meant `amounts to` or `accounts for`.".to_owned(),
                priority: 63,
            });
        }

        None
    }

    fn description(&self) -> &str {
        "Corrects `amounts for` to either `amounts to` or `accounts for`"
    }
}

#[cfg(test)]
mod tests {
    use super::AmountsFor;
    use crate::linting::tests::assert_top3_suggestion_result;

    #[test]
    fn corrects_that_amounts_for_to_amounts_to_entire_value() {
        assert_top3_suggestion_result(
            "Skyler stated the car wash is worth close to $800k, that amounts for the entire value of the company",
            AmountsFor::default(),
            "Skyler stated the car wash is worth close to $800k, that amounts to the entire value of the company",
        );
    }

    #[test]
    fn corrects_that_amounts_for_to_amounts_to_percent() {
        assert_top3_suggestion_result(
            "Together, that amounts for 1157 calls or 60% of the failures.",
            AmountsFor::default(),
            "Together, that amounts to 1157 calls or 60% of the failures.",
        );
    }

    #[test]
    fn corrects_that_amounts_for_to_accounts_for_setting_up() {
        assert_top3_suggestion_result(
            "One solution to this would be to have separate controllers but the amount of code that amounts for setting up, processing and calling",
            AmountsFor::default(),
            "One solution to this would be to have separate controllers but the amount of code that accounts for setting up, processing and calling",
        );
    }

    #[test]
    fn corrects_which_amounts_for_to_accounts_for_16k() {
        assert_top3_suggestion_result(
            "It has an offset of 0xC000 which amounts for the 16k.",
            AmountsFor::default(),
            "It has an offset of 0xC000 which accounts for the 16k.",
        );
    }

    #[test]
    fn corrects_this_amounts_for_to_accounts_for_large_part() {
        assert_top3_suggestion_result(
            "I'm pretty sure that this amounts for a large part of the speed I gained when typing, in addition to touch-typing.",
            AmountsFor::default(),
            "I'm pretty sure that this accounts for a large part of the speed I gained when typing, in addition to touch-typing.",
        );
    }

    #[test]
    fn corrects_they_amount_for_to_amount_to_16kb() {
        assert_top3_suggestion_result(
            "it is obvious that the messages are being held \"somewhere\" until they amount for 16kB and then the whole lot come at once.",
            AmountsFor::default(),
            "it is obvious that the messages are being held \"somewhere\" until they amount to 16kB and then the whole lot come at once.",
        );
    }

    #[test]
    fn corrects_which_amounts_for_to_amounts_to_10_minutes() {
        assert_top3_suggestion_result(
            "set a small TTL for your hostname (like 600 which amounts for 10 minutes).",
            AmountsFor::default(),
            "set a small TTL for your hostname (like 600 which amounts to 10 minutes).",
        );
    }

    #[test]
    fn corrects_it_amounts_for_to_amounts_to_redefinition() {
        assert_top3_suggestion_result(
            "included for convenience to get a Lorentz invariant result (it amounts for a redefinition of ap).",
            AmountsFor::default(),
            "included for convenience to get a Lorentz invariant result (it amounts to a redefinition of ap).",
        );
    }

    #[test]
    fn corrects_they_amount_for_to_amount_to_nothing() {
        assert_top3_suggestion_result(
            "Matter and antimatter are spread throughout the Universe, and in total, they amount for nothing",
            AmountsFor::default(),
            "Matter and antimatter are spread throughout the Universe, and in total, they amount to nothing",
        );
    }

    #[test]
    fn would_amount_for_to_amount_to_api_requests() {
        assert_top3_suggestion_result(
            "10% of 6,782,091 would amount for 678,209 API requests",
            AmountsFor::default(),
            "10% of 6,782,091 would amount to 678,209 API requests",
        );
    }

    #[test]
    fn will_amount_for_to_amount_to_relationships() {
        assert_top3_suggestion_result(
            "Consider this statistic from Gartner, that artificial intelligence will amount for 85% of customer relationships by 2020.",
            AmountsFor::default(),
            "Consider this statistic from Gartner, that artificial intelligence will amount to 85% of customer relationships by 2020.",
        );
    }

    #[test]
    fn should_amount_for_to_amount_to_half_pack() {
        assert_top3_suggestion_result(
            "It doesn't seem realistic that this single elite should amount for half the pack",
            AmountsFor::default(),
            "It doesn't seem realistic that this single elite should amount to half the pack",
        );
    }

    #[test]
    fn can_amount_for_to_amount_to_draw_calls() {
        assert_top3_suggestion_result(
            "That can amount for a lot of draw calls and work for the engine to cull. ",
            AmountsFor::default(),
            "That can amount to a lot of draw calls and work for the engine to cull. ",
        );
    }
}
