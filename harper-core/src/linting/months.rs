use crate::linting::expr_linter::Chunk;
use crate::{
    Lrc, Token, TokenKind,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

// Static array of all month names
const ALL_MONTHS: &[&str] = &[
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

pub struct Months {
    expr: Box<dyn Expr>,
}

impl Default for Months {
    fn default() -> Self {
        // Define ambiguous months (those that are also common words)
        let ambiguous_months = Lrc::new(WordSet::new(&["march", "may", "august"]));

        // The unambiguous months
        let only_months: Vec<&str> = ALL_MONTHS
            .iter()
            .filter(|&&m| !ambiguous_months.contains(m))
            .copied()
            .collect();

        let only_months = WordSet::new(&only_months);

        let before_month_sense_only = WordSet::new(&[
            // Determiners.
            // These words won't disambiguate months: "each", "this", "that"
            // "each may do as he likes"
            // "this may be the best month"
            "every",
            // Prepositions.
            // Possible false positives:
            // "the first word at the beginning of the next may be fragmented"
            // "Next may be to offer all the color tables in some way"
            // "First and last may have been swapped"
            "by", "during", "in", "last", "next", "of", "until",
        ]);

        let year_or_day_of_month = SequenceExpr::default().then_kind_where(|kind| {
            if let TokenKind::Number(number) = &kind {
                let v = number.value.into_inner() as u32;
                (1500..=2500).contains(&v) || (1..=31).contains(&v)
            } else {
                false
            }
        });

        // An Expr that matches either a plain month
        // Or an ambiguous month after a disambiguating word
        let month_expr = SequenceExpr::with(FirstMatchOf::new(vec![
            Box::new(only_months),
            Box::new(
                SequenceExpr::with(before_month_sense_only)
                    .then_whitespace()
                    .then(ambiguous_months.clone()),
            ),
            Box::new(
                SequenceExpr::with(ambiguous_months)
                    .then_whitespace()
                    .then(year_or_day_of_month),
            ),
        ]));

        Self {
            expr: Box::new(month_expr),
        }
    }
}

impl ExprLinter for Months {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], src: &[char]) -> Option<Lint> {
        // `find` which token is the month by seeing which tok's content (lowercased) is in ALL_MONTHS
        let month_tok = tokens.iter().find(|token| {
            let token_str = token.span.get_content_string(src);
            ALL_MONTHS.iter().any(|&m| m == token_str.to_lowercase())
        })?; // Return None if no month token found

        // let month_tok = tokens.last().unwrap();
        let month_ch = month_tok.span.get_content(src);

        if month_ch[0].is_uppercase() {
            return None;
        }

        let mut month_vec = month_ch.to_vec();
        month_vec[0] = month_vec[0].to_ascii_uppercase();

        Some(Lint {
            span: month_tok.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith(month_vec)],
            message: "Months should be written with a capital letter.".to_string(),
            priority: 126,
        })
    }

    fn description(&self) -> &str {
        "Detects months written with a lowercase first letter."
    }
}

#[cfg(test)]
mod tests {
    use super::Months;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn fix_in_august() {
        assert_suggestion_result(
            "I worked for WebstaurantStore doing Quality Assurance Automation and am now transitioning to a new graduate developer role at BNY Mellon, starting in august.",
            Months::default(),
            "I worked for WebstaurantStore doing Quality Assurance Automation and am now transitioning to a new graduate developer role at BNY Mellon, starting in August.",
        );
    }

    #[test]
    fn fix_in_march() {
        assert_suggestion_result(
            "This game was originally written by me in march 2000.",
            Months::default(),
            "This game was originally written by me in March 2000.",
        );
    }

    #[test]
    fn fix_in_may() {
        assert_suggestion_result(
            "typo in may 2024 updates",
            Months::default(),
            "typo in May 2024 updates",
        );
    }

    #[test]
    fn fix_last_august() {
        assert_suggestion_result(
            "since last august smart has been leading talks to open up japan",
            Months::default(),
            "since last August smart has been leading talks to open up japan",
        );
    }

    #[test]
    fn fix_last_may() {
        assert_suggestion_result(
            "I have a 2019 mini countryman that i purchased last may.",
            Months::default(),
            "I have a 2019 mini countryman that i purchased last May.",
        );
    }

    #[test]
    fn fix_of_august() {
        assert_suggestion_result(
            "change abbreviation of august for Indonesian locale",
            Months::default(),
            "change abbreviation of August for Indonesian locale",
        )
    }

    #[test]
    fn fix_march_2019() {
        assert_suggestion_result(
            "How to disable drop cap today (late march 2019)",
            Months::default(),
            "How to disable drop cap today (late March 2019)",
        );
    }

    #[test]
    fn fix_may_2022() {
        assert_suggestion_result(
            "That will be ende from 30 may 2022.",
            Months::default(),
            "That will be ende from 30 May 2022.",
        );
    }

    #[test]
    fn fix_days() {
        assert_suggestion_result(
            "Between march 15 and august 27.",
            Months::default(),
            "Between March 15 and August 27.",
        );
    }
}
