use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

const RARE_BEFORE_A_LONG: &[&str] = &[
    "all", "and", "came", "cut", "distance", "drums", "get", "getting", "go", "it", "located",
    "roll", "section", "shuffle", "sing", "tear", "walked", "went",
];
const RARE_BEFORE_ALONG: &[&str] = &[
    "after", "been", "for", "had", "has", "in", "is", "it's", "of", "over", "was", "with",
];
// Pronoun, Determiner before "along" but not before "a long"
// Adverb before "a long" but not before "along"
const RARE_AFTER_A_LONG: &[&str] = &[
    "a",            // det.indef
    "an",           // det.indef
    "came",
    "dotted",
    "in",           // prep
    "its",          // det.poss
    "one",
    "perforated",
    "said",
    "similar",
    "the",          // det.def
    "these",        // det. / pron.
    "this",         // det. / pron.
    "to",           // prep.
    "with",         // prep.
];
const RARE_AFTER_ALONG: &[&str] = &[
    "day", "history", "life", "list", "moment", "pause", "period", "range", "series", "silence",
    "story", "term", "time", "way", "while",
];
// Determiner, Adverb, Pronoun - after "along" but not after "a long"

pub struct ALongAlong {
    expr: FirstMatchOf,
}

impl Default for ALongAlong {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::word_set(RARE_BEFORE_A_LONG)
                        .t_ws()
                        .then_word_seq(&["a", "long"]),
                ) as Box<dyn Expr>,
                Box::new(
                    SequenceExpr::word_set(RARE_BEFORE_ALONG)
                        .t_ws()
                        .t_aco("along"),
                ),
                Box::new(
                    SequenceExpr::word_seq(&["a", "long"])
                        .t_ws()
                        .t_set(RARE_AFTER_A_LONG),
                ),
                Box::new(SequenceExpr::aco("along").t_ws().t_set(RARE_AFTER_ALONG)),
            ]),
        }
    }
}

impl ExprLinter for ALongAlong {
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
        let message = "Fix this erorr".to_owned();
        Some(Lint {
            span,
            lint_kind,
            suggestions,
            message,
            ..Default::default()
        })
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

    use super::ALongAlong;

    #[test]
    fn test_skeleton() {
        assert_suggestion_result("erorr", ALongAlong::default(), "correction");
    }
}
