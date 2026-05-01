use crate::{
    Dialect, Lint, Token, TokenStringExt,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

static VALID_VARIANTS: &[&[&str]] = &[
    &["worst", "come", "to", "worst"],  // historical original
    &["worst", "comes", "to", "worst"], // ngram-en #1, ngram-us #2, ngram-uk #2
    &["worse", "comes", "to", "worst"], // ngram-en #3, ngram-us #1, ngram-uk #3
    &["the", "worst", "comes", "to", "the", "worst"], // ngram-en #2, ngram-us #3, ngram-uk #1
];

pub struct WorstComesToWorst {
    expr: SequenceExpr,
    dialect: Dialect,
}

impl WorstComesToWorst {
    pub fn new(dialect: Dialect) -> Self {
        Self {
            expr: SequenceExpr::aco("if")
                .t_ws()
                .then_optional(SequenceExpr::aco("the").t_ws())
                .t_set(&["worse", "worst"])
                .t_ws()
                .t_set(&["come", "comes"])
                .t_ws()
                .t_set(&["to", "too"])
                .t_ws()
                .then_optional(SequenceExpr::aco("the").t_ws())
                .t_set(&["worst", "worse"]),
            dialect,
        }
    }
}

impl ExprLinter for WorstComesToWorst {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], source: &[char]) -> Option<Lint> {
        // Return None if we matched any of the valid variants
        let is_valid_variant = VALID_VARIANTS.iter().any(|variant| {
            let mut matched_words_iter = toks
                .iter()
                .skip(2) // Skip "if" and following whitespace
                .enumerate()
                .filter(|(i, _)| *i % 2 == 0) // Even indices (words)
                .map(|(_, token)| token.get_ch(source));

            // All pairs must match and lengths must be equal (no extra matched words)
            variant
                .iter()
                .zip(&mut matched_words_iter)
                .all(|(expected, actual)| actual.eq_str(expected))
                && matched_words_iter.next().is_none()
        });

        if is_valid_variant {
            return None;
        }

        let span = toks.span()?;
        let template = span.get_content(source);

        let suggestions = VALID_VARIANTS
            .iter()
            .map(|words| {
                let word_iter = std::iter::once("if").chain(words.iter().copied());

                let mut replacement = Vec::new();
                for (i, word) in word_iter.enumerate() {
                    if i > 0 {
                        replacement.push(' ');
                    }
                    replacement.extend(word.chars());
                }

                Suggestion::replace_with_match_case(replacement, template)
            })
            .collect::<Vec<_>>();

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "Choose one of these more standard variants of this idiom.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter for wrong variants of (the) worse/worst comes to (the) worst."
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Dialect,
        linting::tests::{assert_no_lints, assert_suggestion_result},
    };

    use super::WorstComesToWorst;

    #[test]
    fn fix_main_us_variant() {
        assert_suggestion_result(
            "if the worst come too the worse",
            WorstComesToWorst::new(Dialect::American),
            "if worse comes to worst",
        );
    }

    #[test]
    fn fix_main_uk_variant() {
        assert_suggestion_result(
            "if the worst comes to worse",
            WorstComesToWorst::new(Dialect::British),
            "if the worst comes to the worst",
        );
    }

    #[test]
    fn dont_flag_historical_original() {
        assert_no_lints(
            "if worst come to worst",
            WorstComesToWorst::new(Dialect::American),
        );
    }

    #[test]
    fn dont_flag_uk_preferred_variant() {
        assert_no_lints(
            "if the worst comes to the worst",
            WorstComesToWorst::new(Dialect::British),
        );
    }

    #[test]
    fn dont_flag_us_preferred_variant() {
        assert_no_lints(
            "if worse comes to worst",
            WorstComesToWorst::new(Dialect::American),
        );
    }

    #[test]
    fn dont_flag_overall_preferred_variant() {
        assert_no_lints(
            "if worst comes to worst",
            WorstComesToWorst::new(Dialect::Indian),
        );
    }
}
