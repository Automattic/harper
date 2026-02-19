use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::NominalPhrase,
};

/// Linter that corrects "take X serious" to "take X seriously".
///
/// This linter identifies and corrects the common mistake of using the adjective "serious"
/// instead of the adverb "seriously" in phrases like "take it serious".
pub struct TakeSerious {
    expr: Box<dyn Expr>,
}

impl Default for TakeSerious {
    /// Creates a new `TakeSerious` instance with the default pattern.
    ///
    /// The pattern matches:
    /// - Any form of "take" (take/takes/taking/took/taken)
    /// - Followed by a nominal phrase
    /// - Ending with "serious"
    fn default() -> Self {
        let pattern = SequenceExpr::word_set(&["take", "taken", "takes", "taking", "took"])
            .t_ws()
            .then(NominalPhrase)
            .t_ws()
            .t_aco("serious");

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for TakeSerious {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let whole_phrase_span = matched_tokens.span()?;
        let all_but_last_token = matched_tokens[..matched_tokens.len() - 1].span()?;

        let mut sugg_value = all_but_last_token.get_content(source).to_vec();
        sugg_value.extend_from_slice(&['s', 'e', 'r', 'i', 'o', 'u', 's', 'l', 'y']);

        let sugg_template = whole_phrase_span.get_content(source);

        let suggestions = vec![Suggestion::replace_with_match_case(
            sugg_value,
            sugg_template,
        )];

        Some(Lint {
            span: whole_phrase_span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message: "Take seriously".to_string(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Ensures the correct use of the adverb `seriously` instead of the adjective `serious` in phrases like `take it seriously`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::TakeSerious;

    #[test]
    fn take_it() {
        assert_suggestion_result(
            "I take it serious.",
            TakeSerious::default(),
            "I take it seriously.",
        );
    }

    #[test]
    #[ignore = "'This' and 'that', which can be determiners and pronouns, are not handled properly by `NominalPhrase`"]
    fn take_this() {
        assert_suggestion_result(
            "What's more important is, that it's impossible to actually take this serious when ...",
            TakeSerious::default(),
            "What's more important is, that it's impossible to actually take this seriously when ...",
        );
    }

    #[test]
    fn not_take_security() {
        assert_suggestion_result(
            "When you say someone does not take security serious you are being judgemental / destructive.",
            TakeSerious::default(),
            "When you say someone does not take security seriously you are being judgemental / destructive.",
        );
    }

    #[test]
    fn we_take_security() {
        assert_suggestion_result(
            "We take security serious.",
            TakeSerious::default(),
            "We take security seriously.",
        );
    }

    #[test]
    fn take_me() {
        assert_suggestion_result(
            "Yeah , don't take me serious , i do this as a hobby - jusspatel.",
            TakeSerious::default(),
            "Yeah , don't take me seriously , i do this as a hobby - jusspatel.",
        );
    }

    #[test]
    #[ignore = "Passive voice and adverbs are not yet supported"]
    fn taken_adv() {
        assert_suggestion_result(
            "This is not meant to be taken overly serious",
            TakeSerious::default(),
            "This is not meant to be taken overly seriously",
        );
    }

    #[test]
    fn takes_these_numbers() {
        assert_suggestion_result(
            "if a program actually takes these numbers serious the results could be catastrophic.",
            TakeSerious::default(),
            "if a program actually takes these numbers seriously the results could be catastrophic.",
        );
    }

    #[test]
    #[ignore = "'No one' is not handled properly by `NominalPhrase`"]
    fn takes_bf() {
        assert_suggestion_result(
            "And obviously no one takes brainfuck serious as a language.",
            TakeSerious::default(),
            "And obviously no one takes brainfuck seriously as a language.",
        );
    }

    #[test]
    #[ignore = "Adverbs are not yet supported"]
    fn taken_very() {
        assert_suggestion_result(
            "Hmm flaky soldering iron is something that must be taken very serious.",
            TakeSerious::default(),
            "Hmm flaky soldering iron is something that must be taken very seriously.",
        );
    }
}
