use crate::{
    CharStringExt, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk},
    token_string_ext::TokenStringExt,
};

pub struct AllIntentsAndPurposes {
    expr: All,
}

impl Default for AllIntentsAndPurposes {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_preposition() // Only "for" or "to" are OK
                .t_ws()
                .t_aco("all")
                .t_ws()
                .then_any_of(vec![
                    Box::new(
                        SequenceExpr::word_set(&[
                            "intents", // Correct, as long as it follows "for" or "to"
                            "extents", "intense", // Incorrect, no matter the preposition
                        ])
                        .t_ws()
                        .t_aco("and"),
                    ),
                    Box::new(SequenceExpr::word_set(&[
                        "intended",
                        "intense",
                        "intensive",
                        "intrinsic",
                    ])),
                ])
                .t_ws()
                .t_set(&["purposes", "purpose"])
                .but_not(
                    SequenceExpr::word_set(&["for", "to"])
                        .t_ws()
                        .then_word_seq(&["all", "intents", "and", "purposes"]),
                ),
        }
    }
}

impl ExprLinter for AllIntentsAndPurposes {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let whole_span = toks.span()?;
        let prep_ch = toks.first()?.get_ch(src);

        let (rest_lo, rest_up) = (" all intents and purposes", " ALL INTENTS AND PURPOSES");

        let (for_prefix, to_prefix, rest) = match (prep_ch.first(), prep_ch.get(1)) {
            (Some(f), Some(s)) if f.is_uppercase() && s.is_uppercase() => ("FOR", "TO", rest_up),
            (Some(f), _) if f.is_uppercase() => ("For", "To", rest_lo),
            _ => ("for", "to", rest_lo),
        };

        let is_to = prep_ch.starts_with_ignore_ascii_case_str("to");

        let build_sugg =
            |prefix: &str| -> Vec<char> { prefix.chars().chain(rest.chars()).collect() };

        let suggestions = if is_to {
            vec![build_sugg(to_prefix), build_sugg(for_prefix)]
        } else {
            vec![build_sugg(for_prefix), build_sugg(to_prefix)]
        }
        .into_iter()
        .map(Suggestion::ReplaceWith)
        .collect();

        let message = format!(
            "The correct form is '{} all intents and purposes'.",
            prep_ch
                .iter()
                .map(|c| c.to_ascii_lowercase())
                .collect::<String>()
        );

        Some(Lint {
            span: whole_span,
            lint_kind: LintKind::Nonstandard,
            suggestions,
            message,
            priority: 50,
        })
    }

    fn description(&self) -> &'static str {
        "Finds and corrects common wrong forms of the phrase 'for all intents and purposes' / 'to all intents and purposes'."
    }
}

#[cfg(test)]
mod tests {
    use super::AllIntentsAndPurposes;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    // Adjectives without "and"

    #[test]
    fn fix_for_intended() {
        assert_suggestion_result(
            "The details tag should be treated like a div for all intended purposes, but unsure where in the selection logic",
            AllIntentsAndPurposes::default(),
            "The details tag should be treated like a div for all intents and purposes, but unsure where in the selection logic",
        );
    }

    #[test]
    fn fix_for_intense() {
        assert_suggestion_result(
            "For all intense purposes, I thought your code was really well written",
            AllIntentsAndPurposes::default(),
            "For all intents and purposes, I thought your code was really well written",
        );
    }

    #[test]
    fn fix_for_intensive() {
        assert_suggestion_result(
            "MultiNode could be for all intensive purposes, the same as Node sans the way it creates the command line arguments for the process.",
            AllIntentsAndPurposes::default(),
            "MultiNode could be for all intents and purposes, the same as Node sans the way it creates the command line arguments for the process.",
        );
    }

    #[test]
    fn fix_for_intrinsic_purposes() {
        assert_suggestion_result(
            "For all intrinsic purposes I think you are wrong.",
            AllIntentsAndPurposes::default(),
            "For all intents and purposes I think you are wrong.",
        );
    }

    #[test]
    fn fix_in_intense_purposes() {
        assert_suggestion_result(
            "the solution has some rules than in all intense purposes is not necessarily database driven",
            AllIntentsAndPurposes::default(),
            "the solution has some rules than for all intents and purposes is not necessarily database driven",
        );
    }

    #[test]
    fn fix_to_intensive_purposes() {
        assert_suggestion_result(
            "To all intensive purposes, for the consumer, a view is a table",
            AllIntentsAndPurposes::default(),
            "To all intents and purposes, for the consumer, a view is a table",
        );
    }

    // Nouns with "and"

    #[test]
    fn fix_at_intents_and() {
        assert_suggestion_result(
            "can be thought of, at all intents and purposes, as a controlled cache",
            AllIntentsAndPurposes::default(),
            "can be thought of, for all intents and purposes, as a controlled cache",
        );
    }

    #[test]
    fn fix_by_intents_and() {
        assert_suggestion_result(
            "so by all intents and purposes one is not nested into another",
            AllIntentsAndPurposes::default(),
            "so for all intents and purposes one is not nested into another",
        );
    }

    #[test]
    fn fix_for_extents_and() {
        assert_suggestion_result(
            "#include, for all extents and purposes (if you take the preprocessor out) just copies the file",
            AllIntentsAndPurposes::default(),
            "#include, for all intents and purposes (if you take the preprocessor out) just copies the file",
        );
    }

    #[test]
    fn dont_flag_for_intents_and() {
        assert_no_lints(
            "with the previous previous setting still present and for all intents and purposes seems enabled",
            AllIntentsAndPurposes::default(),
        );
    }

    #[test]
    fn fix_from_intents_and() {
        assert_suggestion_result(
            "act as a full archive node from all intents and purposes",
            AllIntentsAndPurposes::default(),
            "act as a full archive node for all intents and purposes",
        );
    }

    #[test]
    fn fix_in_intents_and() {
        assert_suggestion_result(
            "I posted #20493 asking about, in all intents and purposes, deno info",
            AllIntentsAndPurposes::default(),
            "I posted #20493 asking about, for all intents and purposes, deno info",
        );
    }

    #[test]
    fn fix_on_intents_and() {
        assert_suggestion_result(
            "It depends on all intents and purposes what you want to do.",
            AllIntentsAndPurposes::default(),
            "It depends for all intents and purposes what you want to do.",
        );
    }

    #[test]
    fn fix_through_intents_and() {
        assert_suggestion_result(
            "While, I know through all intents and purposes it is an ugly url to look at",
            AllIntentsAndPurposes::default(),
            "While, I know for all intents and purposes it is an ugly url to look at",
        );
    }

    #[test]
    fn fix_to_extents_and() {
        assert_suggestion_result(
            "and they were trying to find out how that would also affect the the personnel to all extents and purposes.",
            AllIntentsAndPurposes::default(),
            "and they were trying to find out how that would also affect the the personnel to all intents and purposes.",
        );
    }

    #[test]
    fn dont_flag_to_intents_and() {
        assert_no_lints(
            "and they were trying to find out how that would also affect the the personnel to all intents and purposes.",
            AllIntentsAndPurposes::default(),
        );
    }

    #[test]
    fn fix_with_intents_and() {
        assert_suggestion_result(
            "With all intents and purposes the array should be As String since all values I'll be dealing with will be strings.",
            AllIntentsAndPurposes::default(),
            "For all intents and purposes the array should be As String since all values I'll be dealing with will be strings.",
        );
    }

    // Adjectives with "and"!

    #[test]
    fn fix_by_intensive_purposes() {
        assert_suggestion_result(
            "By all intensive purposes this should be working",
            AllIntentsAndPurposes::default(),
            "For all intents and purposes this should be working",
        );
    }

    #[test]
    fn fix_for_intense_and() {
        assert_suggestion_result(
            "to test my site, which for all intense and purposes works",
            AllIntentsAndPurposes::default(),
            "to test my site, which for all intents and purposes works",
        );
    }

    #[test]
    fn fix_in_intensive_purposes() {
        assert_suggestion_result(
            "it should in all intensive purposes keep running",
            AllIntentsAndPurposes::default(),
            "it should for all intents and purposes keep running",
        );
    }

    #[test]
    fn fix_to_intense_and() {
        assert_suggestion_result(
            "The other type, is to all intense and purposes a submit button to the browser",
            AllIntentsAndPurposes::default(),
            "The other type, is to all intents and purposes a submit button to the browser",
        );
    }

    // Doesn't try to deal with qualified "all"

    #[test]
    fn dont_flag_for_basically_all_intents_and_purposes() {
        assert_no_lints(
            "For basically all intents and purposes, this works fine.",
            AllIntentsAndPurposes::default(),
        );
    }

    #[test]
    fn dont_flag_for_nearly_all_intents_and_purposes() {
        assert_no_lints(
            "but for nearly all intents and purposes this should be negligable",
            AllIntentsAndPurposes::default(),
        );
    }

    #[test]
    fn dont_flag_for_pretty_much_all_intents_and_purposes() {
        assert_no_lints(
            "or for pretty much all intents and purposes, between Android devices",
            AllIntentsAndPurposes::default(),
        );
    }

    // Strange false positives

    #[test]
    #[ignore = "Rare and unusual false positive"]
    fn false_positive_for_99_percent_of_all_intents_and_purposes() {
        assert_no_lints(
            "But for 99% of all intents and purposes they can be treated as lists.",
            AllIntentsAndPurposes::default(),
        );
    }

    // US Constitution false positive

    #[test]
    fn false_positive_for_us_constitution_space() {
        assert_no_lints(
            "Amendments, which, in either Case, shall be valid to all Intents and Purposes, as Part of this Constitution",
            AllIntentsAndPurposes::default(),
        );
    }

    #[test]
    #[ignore = "The linefeed should be treated as a space!"]
    fn false_positive_for_us_constitution_line_break() {
        assert_no_lints(
            "Amendments, which, in either Case, shall be valid to all Intents and\nPurposes, as Part of this Constitution",
            AllIntentsAndPurposes::default(),
        );
    }

    #[test]
    fn fix_all_intents_and_purpose() {
        assert_suggestion_result(
            "To all intents and purpose, the SEC no longer exists",
            AllIntentsAndPurposes::default(),
            "To all intents and purposes, the SEC no longer exists",
        );
    }

    #[test]
    fn dont_flag_constitution() {
        assert_suggestion_result(
            "Amendments, which, in either Case, shall be valid to all Intents and Purposes, as Part of this Constitution",
            AllIntentsAndPurposes::default(),
            "Amendments, which, in either Case, shall be valid to all Intents and Purposes, as Part of this Constitution",
        );
    }
}
