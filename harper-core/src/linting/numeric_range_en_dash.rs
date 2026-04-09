use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::punctuation::Punctuation;

use super::{ExprLinter, Lint, LintKind, Suggestion};

const EN_DASH: char = '–';

fn is_target_dash(token: &Token, _source: &[char]) -> bool {
    matches!(
        token.kind.as_punctuation(),
        Some(Punctuation::Hyphen | Punctuation::EmDash)
    )
}

fn is_numeric_range_dash(token: &Token) -> bool {
    matches!(
        token.kind.as_punctuation(),
        Some(Punctuation::Hyphen | Punctuation::EnDash | Punctuation::EmDash)
    )
}

fn is_chained_numeric_form(context: Option<(&[Token], &[Token])>) -> bool {
    let Some((before, after)) = context else {
        return false;
    };

    // Skip multipart numeric chains like dates (`2026-03-18`) and versions (`1-2-3`).
    let preceded_by_numeric_dash = matches!(
        before,
        [.., number, dash] if number.kind.is_number() && is_numeric_range_dash(dash)
    );

    let followed_by_dash_numeric = matches!(
        after,
        [dash, number, ..] if is_numeric_range_dash(dash) && number.kind.is_number()
    );

    preceded_by_numeric_dash || followed_by_dash_numeric
}

pub struct NumericRangeEnDash {
    expr: SequenceExpr,
}

impl Default for NumericRangeEnDash {
    fn default() -> Self {
        // Match isolated numeric ranges like `12-14` or `3—5`.
        // The context check below skips dates, version chains, and similar
        // multipart numeric forms that should keep their existing separators.
        let pattern = SequenceExpr::default()
            .then_number()
            .then(is_target_dash as fn(&Token, &[char]) -> bool)
            .then_number();

        Self { expr: pattern }
    }
}

impl ExprLinter for NumericRangeEnDash {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        _source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if is_chained_numeric_form(context) {
            return None;
        }

        Some(Lint {
            span: matched_tokens[1].span,
            lint_kind: LintKind::Formatting,
            suggestions: vec![Suggestion::ReplaceWith(vec![EN_DASH])],
            message: "Use an en dash (–) in ranges of numbers. Ignore this if it is math."
                .to_owned(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Replaces hyphens and em dashes with en dashes in isolated numeric ranges such as `12–14`."
    }
}

#[cfg(test)]
mod tests {
    use super::NumericRangeEnDash;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn corrects_basic_page_range() {
        assert_suggestion_result(
            "See pages 12-14 for the full table.",
            NumericRangeEnDash::default(),
            "See pages 12–14 for the full table.",
        );
    }

    #[test]
    fn corrects_basic_em_dash_range() {
        assert_suggestion_result(
            "Read chapters 3—5 before class.",
            NumericRangeEnDash::default(),
            "Read chapters 3–5 before class.",
        );
    }

    #[test]
    fn corrects_score_range() {
        assert_suggestion_result(
            "The final score was 10-8 after overtime.",
            NumericRangeEnDash::default(),
            "The final score was 10–8 after overtime.",
        );
    }

    #[test]
    fn corrects_decimal_range() {
        assert_suggestion_result(
            "Keep the ratio between 1.5-2.5 during calibration.",
            NumericRangeEnDash::default(),
            "Keep the ratio between 1.5–2.5 during calibration.",
        );
    }

    #[test]
    fn corrects_percent_range() {
        assert_suggestion_result(
            "Expect a 5-10% improvement after tuning.",
            NumericRangeEnDash::default(),
            "Expect a 5–10% improvement after tuning.",
        );
    }

    #[test]
    fn corrects_year_span() {
        assert_suggestion_result(
            "The archive covers 1990-1995.",
            NumericRangeEnDash::default(),
            "The archive covers 1990–1995.",
        );
    }

    #[test]
    fn corrects_zero_padded_range() {
        assert_suggestion_result(
            "Use files 01-03 for the demo.",
            NumericRangeEnDash::default(),
            "Use files 01–03 for the demo.",
        );
    }

    #[test]
    fn corrects_sentence_final_range() {
        assert_suggestion_result(
            "Valid values are 2-4.",
            NumericRangeEnDash::default(),
            "Valid values are 2–4.",
        );
    }

    #[test]
    fn ignores_existing_en_dash() {
        assert_no_lints(
            "See pages 12–14 for the full table.",
            NumericRangeEnDash::default(),
        );
    }

    #[test]
    fn ignores_spaced_hyphen_range() {
        assert_no_lints(
            "See pages 12 - 14 for the full table.",
            NumericRangeEnDash::default(),
        );
    }

    #[test]
    fn ignores_spaced_em_dash_range() {
        assert_no_lints(
            "See pages 12 — 14 for the full table.",
            NumericRangeEnDash::default(),
        );
    }

    #[test]
    fn ignores_hyphenated_date_chain() {
        assert_no_lints("Today is 2026-03-18.", NumericRangeEnDash::default());
    }

    #[test]
    fn ignores_em_dash_date_chain() {
        assert_no_lints("Today is 2026—03—18.", NumericRangeEnDash::default());
    }

    #[test]
    fn ignores_three_part_version_chain() {
        assert_no_lints("The build number is 1-2-3.", NumericRangeEnDash::default());
    }

    #[test]
    fn ignores_mixed_dash_chain() {
        assert_no_lints("The timeline reads 1-2—3.", NumericRangeEnDash::default());
    }

    #[test]
    fn ignores_longer_numeric_chain() {
        assert_no_lints(
            "The code spans 12-14-16 in the export.",
            NumericRangeEnDash::default(),
        );
    }
}
