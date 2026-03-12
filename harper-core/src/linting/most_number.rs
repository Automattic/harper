use crate::expr::All;
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct MostNumber {
    expr: Box<dyn Expr>,
}

impl Default for MostNumber {
    fn default() -> Self {
        Self {
            expr: Box::new(All::new(vec![
                // Main pattern
                Box::new(
                    SequenceExpr::default()
                        .t_aco("most")
                        .t_ws()
                        .then_word_set(&["amount", "number"]),
                ),
                // Context pattern
                Box::new(
                    SequenceExpr::anything()
                        .then_anything()
                        .then_anything()
                        .then_anything()
                        .t_aco("of"),
                ),
            ])),
        }
    }
}

impl ExprLinter for MostNumber {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], source: &[char]) -> Option<Lint> {
        let most_amt_num_span = toks[0..3].span()?;
        let noun_string = toks[2].span.get_content_string(source);
        let superlatives = if noun_string == "amount" {
            vec!["largest", "greatest", "maximum"]
        } else {
            vec!["highest", "largest", "maximum"]
        };
        let suggestions = superlatives
            .into_iter()
            .map(|superlative| {
                Suggestion::replace_with_match_case(
                    format!("{superlative} {noun_string}").chars().collect(),
                    most_amt_num_span.get_content(source),
                )
            })
            .collect();
        Some(Lint {
            span: most_amt_num_span,
            lint_kind: LintKind::Miscellaneous,
            suggestions,
            message: format!(
                "`Most` is not standard before `{}`.",
                toks[2].span.get_content_string(source)
            ),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Corrects `most number` and `most amount`"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_lint_count, assert_suggestion_result, assert_top3_suggestion_result,
    };

    use super::MostNumber;

    #[test]
    fn corrects_most_number() {
        assert_suggestion_result(
            "Find artists that have been on Spotify the most number of times.",
            MostNumber::default(),
            "Find artists that have been on Spotify the highest number of times.",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case currently produces 'GreatEst'"]
    fn corrects_most_amount_title_case() {
        assert_top3_suggestion_result(
            "Area of Container with the Most Amount of Water",
            MostNumber::default(),
            "Area of Container with the Greatest Amount of Water",
        );
    }

    #[test]
    fn corrects_most_amount() {
        assert_top3_suggestion_result(
            "I just wanted to make sure it's good for the most amount of people, not just what I like.",
            MostNumber::default(),
            "I just wanted to make sure it's good for the greatest amount of people, not just what I like.",
        );
    }

    #[test]
    fn dont_correct_most_number_without_context() {
        assert_lint_count(
            "The random non-sequential nature should prevent most number gaming/sniping/lunging.",
            MostNumber::default(),
            0,
        );
    }

    #[test]
    fn corrects_most_amount_with_maximum() {
        assert_top3_suggestion_result(
            "If you want to support the most amount of different architectures ...",
            MostNumber::default(),
            "If you want to support the maximum amount of different architectures ...",
        );
    }
}
