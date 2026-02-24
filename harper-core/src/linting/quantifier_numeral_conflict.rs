use crate::expr::{All, Expr, SequenceExpr, SpelledNumberExpr};
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, LintKind, Suggestion};
use crate::patterns::{NominalPhrase, WordSet};
use crate::token_string_ext::TokenStringExt;
use crate::{CharStringExt, Lint, Token};

pub struct QuantifierNumeralConflict {
    expr: Box<dyn Expr>,
}

impl Default for QuantifierNumeralConflict {
    fn default() -> Self {
        Self {
            expr: Box::new(All::new(vec![
                Box::new(
                    SequenceExpr::default()
                        .then_quantifier()
                        .t_ws()
                        .then_longest_of(vec![
                            Box::new(SpelledNumberExpr),
                            Box::new(SequenceExpr::default().then_cardinal_number()),
                        ]),
                ),
                Box::new(SequenceExpr::unless(SequenceExpr::any_of(vec![
                    Box::new(WordSet::new(&["all", "any", "every", "no"])),
                    Box::new(
                        SequenceExpr::word_set(&["each", "some"])
                            .t_ws()
                            .t_aco("one"),
                    ),
                ]))),
            ])),
        }
    }
}

impl ExprLinter for QuantifierNumeralConflict {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // If there's a hyphen straight after the number it's probably part of a compound
        if let Some((_, [next_tok, ..])) = ctx
            && next_tok.kind.is_hyphen()
        {
            return None;
        }

        let qtok = toks.first().unwrap();
        let quant = qtok.span.get_content_string(src);

        // Handle special cases for "least", "most", "each", and "both"
        match quant.to_ascii_lowercase().as_str() {
            "least" | "most" => {
                if let Some((previous, _)) = ctx
                    && let [.., prev_word, prev_space] = previous
                    && prev_space.kind.is_whitespace()
                    && prev_word.kind.is_word()
                    && prev_word
                        .span
                        .get_content(src)
                        .eq_ignore_ascii_case_chars(&['a', 't'])
                {
                    return None;
                }
            }

            "each" => {
                return Some(Lint {
                    span: qtok.span,
                    lint_kind: LintKind::Usage,
                    suggestions: vec![Suggestion::replace_with_match_case(
                        "every".chars().collect(),
                        qtok.span.get_content(src),
                    )],
                    message: "Use 'every' instead of 'each' before a number.".to_owned(),
                    ..Default::default()
                });
            }

            "both" => {
                if let Some((_, following)) = ctx
                    && let Some(noun_phrase_span) = NominalPhrase.run(1, following, src)
                    && let [ws, conj, ..] = following.get(noun_phrase_span.end..).unwrap_or(&[])
                    && ws.kind.is_whitespace()
                    && conj.kind.is_conjunction()
                    && conj
                        .span
                        .get_content_string(src)
                        .eq_ignore_ascii_case("and")
                {
                    return None;
                }
            }

            _ => {} // Continue with the default case
        }

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Grammar,
            suggestions: vec![],
            message: format!("The word '{quant}' should not be used before a number."),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects quantifier-numeral conflicts"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    use super::QuantifierNumeralConflict;

    #[test]
    fn flag_several_two() {
        assert_lint_count(
            "A few minutes ago, there was an outage due to several two hosts being down at the same time.",
            QuantifierNumeralConflict::default(),
            1,
        );
    }

    #[test]
    fn dont_flag_at_least() {
        assert_no_lints(
            "Serving a company that encourages the \"996\" work schedule usually means working for at least 60 hours per week.",
            QuantifierNumeralConflict::default(),
        );
    }

    #[test]
    fn dont_flag_at_most() {
        assert_no_lints(
            "But don't worry, the second machine takes at most 3 years.",
            QuantifierNumeralConflict::default(),
        );
    }

    #[test]
    fn dont_flag_both_32_bit_and_64_bit() {
        assert_no_lints(
            "Both 32 bit and 64 bit architectures are supported.",
            QuantifierNumeralConflict::default(),
        );
    }

    #[test]
    fn dont_flag_more_1_click() {
        assert_no_lints(
            "For more 1-click cloud deployments, see [Cloud Deployment",
            QuantifierNumeralConflict::default(),
        );
    }

    #[test]
    fn correct_each_2() {
        assert_suggestion_result(
            "OSSEC by default run rootkit check each 2 hours.",
            QuantifierNumeralConflict::default(),
            "OSSEC by default run rootkit check every 2 hours.",
        );
    }

    #[test]
    fn ignore_no_two_adjacent_characters_2486() {
        assert_no_lints(
            "No two adjacent characters are the same.",
            QuantifierNumeralConflict::default(),
        );
    }
}
