use crate::{
    Lint, Lrc, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct AlludeVsElude {
    expr: SequenceExpr,
}

const COMMON_BEFORE_VERB: &[&str] = &[
    "above", "after", "already", "also", "always", "and", "be", "been", "before", "by", "can",
    "for", "had", "has", "have", "he", "here", "hitherto", "I", "in", "In", "is", "it", "just",
    "may", "not", "now", "of", "often", "only", "or", "probably", "she", "still", "that", "they",
    "This", "thus", "to", "To", "was", "we", "We", "which", "who", "will", "without", "you",
];

const COMMON_BEFORE_BOTH: &[&str] = &[
    "and", "by", "have", "it", "may", "not", "of", "to", "was", "which",
];

const COMMON_AFTER_VERB: &[&str] = &[
    "a",
    "above",
    "all",
    "are",
    "as",
    "at",
    "by",
    "had",
    "her",
    "here",
    "him",
    "his",
    "in",
    "is",
    "it",
    "its",
    "me",
    "more",
    "my",
    "not",
    "only",
    "our",
    "particularly",
    "perhaps",
    "probably",
    "the",
    "their",
    "them",
    "to",
    "us",
    "was",
    "were",
    "when",
    "with",
];

const ALLUDE: &[&str] = &["allude", "alluded", "alludes", "alluding"];
const ELUDE: &[&str] = &["elude", "eluded", "eludes", "eluding"];

impl Default for AlludeVsElude {
    fn default() -> Self {
        let either_verb = Lrc::new(FirstMatchOf::new(vec![
            Box::new(SequenceExpr::word_set(ALLUDE)),
            Box::new(SequenceExpr::word_set(ELUDE)),
        ]));

        Self {
            expr: SequenceExpr::any_of(vec![
                Box::new(
                    SequenceExpr::word_set(COMMON_BEFORE_VERB)
                        .t_ws()
                        .then(either_verb.clone()),
                ),
                Box::new(
                    SequenceExpr::with(either_verb)
                        .t_ws()
                        .t_set(COMMON_AFTER_VERB),
                ),
            ]),
        }
    }
}

impl ExprLinter for AlludeVsElude {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));
        let span = matched_tokens.span()?;
        let lint_kind = LintKind::Miscellaneous;
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "correction",
            span.get_content(source),
        )];
        let message = "Fix this erorr".to_string();
        None /*Some(Lint {
        span,
        lint_kind,
        suggestions,
        message,
        ..Default::default()
        })*/
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter skeleton for contributors to copy into `harper_core/src/linting/` and rename."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::AlludeVsElude;

    #[test]
    fn test_skeleton() {
        assert_suggestion_result("erorr", AlludeVsElude::default(), "correction");
    }
}
