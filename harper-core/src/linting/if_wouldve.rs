use crate::expr::{Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, LintKind, Suggestion};
use crate::patterns::{NominalPhrase, WordSet};
use crate::token_string_ext::TokenStringExt;
use crate::{CharStringExt, Lint, Token};

pub struct IfWouldve {
    expr: Box<dyn Expr>,
}

impl Default for IfWouldve {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::aco("if")
                    .t_ws()
                    .then(NominalPhrase)
                    .t_ws()
                    .then_any_of(vec![
                        Box::new(
                            SequenceExpr::word_set(&["would", "had"])
                                .t_ws()
                                .then_word_set(&["have", "of"]),
                        ),
                        Box::new(WordSet::new(&["would've", "wouldve", "had've", "hadve"])),
                    ])
                    .t_ws()
                    .then_verb_past_participle_form(),
            ),
        }
    }
}

impl ExprLinter for IfWouldve {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    /// Identifies and corrects incorrect conditional phrases like "would've", "would have", "would of", etc.
    /// to use the correct "had" construction in conditional statements.
    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // We examine tokens in pairs, moving backwards from the end of the phrase
        // to find the incorrect verb construction to replace with "had".
        // The pattern we're looking for is: [if] [NP] [would/had] [have/of/ve] [verb]

        // Start from the end of the token sequence (before the verb)
        let matched_tokens = (2..toks.len() - 2)
            .rev()
            .step_by(2) // Check every other token since we're looking at pairs
            .find_map(|i| {
                let prev = toks[i - 2].span.get_content(src);
                let curr = toks[i].span.get_content(src);

                let would_had = &["would", "had"];

                // Determine which tokens to replace based on the pattern
                match () {
                    // Handle contractions like "would've" or "had've"
                    _ if curr.ends_with_ignore_ascii_case_str("ve") => {
                        if curr.starts_with_any_ignore_ascii_case_str(would_had) {
                            Some(&toks[i..=i]) // Single token like "would've"
                        } else if prev.starts_with_any_ignore_ascii_case_str(would_had) {
                            Some(&toks[i - 2..=i]) // Two tokens like "would have"
                        } else {
                            None
                        }
                    }
                    // Handle "would of" / "had of"
                    _ if curr.ends_with_ignore_ascii_case_str("of")
                        && prev.starts_with_any_ignore_ascii_case_str(would_had) =>
                    {
                        Some(&toks[i - 2..=i])
                    }
                    _ => None,
                }
            });

        matched_tokens.and_then(|tokens_to_replace| {
            let span = tokens_to_replace.span()?;

            Some(Lint {
                span,
                lint_kind: LintKind::Nonstandard,
                suggestions: vec![Suggestion::replace_with_match_case(
                    vec!['h', 'a', 'd'],
                    span.get_content(src),
                )],
                message: "If this is counterfactual or hypothetical, use `had` after `if` rather than `would have` or `had have`.".to_string(),
                ..Default::default()
            })
        })
    }

    fn description(&self) -> &str {
        "Corrects `if I would've done` etc. to `if I had done` etc."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::*;

    #[test]
    fn flag_if_i_wouldve_done_x() {
        assert_suggestion_result(
            "If I would've done X...",
            IfWouldve::default(),
            "If I had done X...",
        );
    }

    #[test]
    fn flag_if_you_would_have_done_y() {
        assert_suggestion_result(
            "If you would have done Y...",
            IfWouldve::default(),
            "If you had done Y...",
        );
    }

    #[test]
    fn flag_if_we_would_of_z() {
        assert_suggestion_result(
            "If we would of done Z...",
            IfWouldve::default(),
            "If we had done Z...",
        );
    }

    #[test]
    fn flag_if_he_hadve_done_w() {
        assert_suggestion_result(
            "If he hadve done W...",
            IfWouldve::default(),
            "If he had done W...",
        );
    }

    #[test]
    fn flag_if_she_hadve_done_x() {
        assert_suggestion_result(
            "If she had've done X...",
            IfWouldve::default(),
            "If she had done X...",
        );
    }

    #[test]
    fn flag_if_it_had_of_done_x() {
        assert_suggestion_result(
            "If it had of done X...",
            IfWouldve::default(),
            "If it had done X...",
        );
    }

    #[test]
    fn flag_if_np_wouldve() {
        assert_suggestion_result(
            "If that guy would've thought it through...",
            IfWouldve::default(),
            "If that guy had thought it through...",
        );
    }

    // The linter cannot yet detect when this pattern is a counterfactual.
    // If you can improve this linter to do so, here are some example sentences.

    #[test]
    #[ignore = "Can't detect correct use not in counterfactual"]
    fn dont_flag_non_counterfactual_done() {
        assert_no_lints(
            "I don't know if they would have done that for a designer",
            IfWouldve::default(),
        );
    }

    #[test]
    #[ignore = "Can't detect correct use not in counterfactual"]
    fn dont_flag_non_counterfactual_gotten() {
        assert_no_lints(
            "I don't know if a normal programmer would have gotten that treatment.",
            IfWouldve::default(),
        );
    }

    #[test]
    #[ignore = "Can't detect correct use not in counterfactual"]
    fn dont_flag_non_counterfactual_been() {
        assert_no_lints(
            "I don't know if they would have been interested anyway",
            IfWouldve::default(),
        );
    }
}
