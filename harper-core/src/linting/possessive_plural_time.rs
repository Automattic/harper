use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct PossessivePluralTime {
    expr: SequenceExpr,
}

impl Default for PossessivePluralTime {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set([
                // regular time units
                "microsecond's",
                "nanosecond's",
                "second's",
                "minute's",
                "hour's",
                "day's",
                "week's",
                "month's",
                "year's",
                "decade's",
                "century's",
                // misc time "units"
                "moment's",
                "weekend's",
            ]),
        }
    }
}

impl ExprLinter for PossessivePluralTime {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));
        let span = matched_tokens.span()?;

        let mut correction: Vec<char> = span.get_content(source).to_vec();
        let l = correction.len();
        correction.swap(l - 2, l - 1);

        let suggestions = vec![Suggestion::replace_with_match_case(
            correction,
            span.get_content(source),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::Punctuation,
            suggestions,
            message: "Did you mean the plural possessive?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects singular possessive time units (`hour’s`) where plural possessive (`hours’`) is correct."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::PossessivePluralTime;

    #[test]
    fn test_skeleton() {
        assert_suggestion_result(
            "I lost many hour's of work",
            PossessivePluralTime::default(),
            "I lost many hours' of work",
        );
    }
}
