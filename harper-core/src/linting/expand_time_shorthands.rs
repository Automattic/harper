use crate::expr::Expr;
use crate::expr::SequenceExpr;
use std::sync::Arc;

use super::{ExprLinter, Lint, LintKind};
use crate::Token;
use crate::linting::Suggestion;
use crate::linting::expr_linter::Chunk;
use crate::patterns::{ImpliesQuantity, WordSet};

pub struct ExpandTimeShorthands {
    expr: Box<dyn Expr>,
}

impl ExpandTimeShorthands {
    pub fn new() -> Self {
        let hotwords = Arc::new(WordSet::new(&[
            "hr", "hrs", "min", "mins", "sec", "secs", "ms", "msec", "msecs",
        ]));

        Self {
            expr: Box::new(SequenceExpr::with(ImpliesQuantity).then_longest_of(vec![
                Box::new(SequenceExpr::with(hotwords.clone())),
                Box::new(SequenceExpr::default().t_ws_h().then(hotwords.clone())),
            ])),
        }
    }

    fn get_replacement(abbreviation: &str, plural: Option<bool>) -> Option<&'static str> {
        let is_plural = plural.unwrap_or(matches!(abbreviation, "hrs" | "mins" | "secs" | "msecs"));
        match abbreviation {
            "hr" | "hrs" => Some(if is_plural { "hours" } else { "hour" }),
            "min" | "mins" => Some(if is_plural { "minutes" } else { "minute" }),
            "sec" | "secs" => Some(if is_plural { "seconds" } else { "second" }),
            "ms" | "msec" | "msecs" => Some(if is_plural {
                "milliseconds"
            } else {
                "millisecond"
            }),
            _ => None,
        }
    }
}

impl Default for ExpandTimeShorthands {
    fn default() -> Self {
        Self::new()
    }
}

impl ExprLinter for ExpandTimeShorthands {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_span = matched_tokens.last()?.span;
        let implies_plural = ImpliesQuantity::implies_plurality(matched_tokens.first()?, source);

        let offending_text = offending_span.get_content(source);

        let replacement =
            Self::get_replacement(&offending_text.iter().collect::<String>(), implies_plural)?;

        let mut replacement_chars = Vec::new();

        // If there isn't spacing, insert a space
        if matched_tokens.len() == 2 {
            replacement_chars.push(' ');
        }

        replacement_chars.extend(replacement.chars());

        if replacement_chars == offending_text {
            return None;
        }

        Some(Lint {
            span: offending_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith(replacement_chars)],
            message: format!("Did you mean `{replacement}`?"),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Expands time-related abbreviations (`hr`, `hrs`, `min`, `mins`, `sec`, `secs`, `ms`, `msec`, `msecs`) to their full forms (`hour`, `hours`, `minute`, `minutes`, `second`, `seconds`, `millisecond`, `milliseconds`)."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::ExpandTimeShorthands;

    #[test]
    fn detects_singular_hour() {
        assert_suggestion_result("5 hr", ExpandTimeShorthands::new(), "5 hours");
    }

    #[test]
    fn detects_singular_minute() {
        assert_suggestion_result("10 min", ExpandTimeShorthands::new(), "10 minutes");
    }

    #[test]
    fn detects_singular_second() {
        assert_suggestion_result("30 sec", ExpandTimeShorthands::new(), "30 seconds");
    }

    #[test]
    fn detects_plural_hours() {
        assert_suggestion_result("5 hrs", ExpandTimeShorthands::new(), "5 hours");
    }

    #[test]
    fn detects_plural_minutes() {
        assert_suggestion_result("10 mins", ExpandTimeShorthands::new(), "10 minutes");
    }

    #[test]
    fn detects_plural_seconds() {
        assert_suggestion_result("30 secs", ExpandTimeShorthands::new(), "30 seconds");
    }

    #[test]
    fn detects_millisecond() {
        assert_suggestion_result("5 ms", ExpandTimeShorthands::new(), "5 milliseconds");
    }

    #[test]
    fn detects_milliseconds() {
        assert_suggestion_result("10 msecs", ExpandTimeShorthands::new(), "10 milliseconds");
    }

    #[test]
    fn handles_punctuation_hour() {
        assert_suggestion_result("5 hr.", ExpandTimeShorthands::new(), "5 hours.");
    }

    #[test]
    fn handles_punctuation_minute() {
        assert_suggestion_result("10 min,", ExpandTimeShorthands::new(), "10 minutes,");
    }

    #[test]
    fn handles_punctuation_second() {
        assert_suggestion_result("30 sec!", ExpandTimeShorthands::new(), "30 seconds!");
    }

    #[test]
    fn handles_adjacent_number_hour() {
        assert_suggestion_result("5hr", ExpandTimeShorthands::new(), "5 hours");
    }

    #[test]
    fn handles_adjacent_number_minute() {
        assert_suggestion_result("10-min", ExpandTimeShorthands::new(), "10-minutes");
    }

    #[test]
    fn handles_adjacent_number_second() {
        assert_suggestion_result("30sec", ExpandTimeShorthands::new(), "30 seconds");
    }
}
