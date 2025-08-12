use crate::Token;

use super::Step;

/// A [`Step`] which will match only if the cursor is over the last non-whitespace character in stream.
/// It will return that token.
///
/// For example, if you built `SequenceExpr::default().t_aco("word").then(AnchorEnd)` and ran it on `This is a word`, the resulting `Span` would only cover the final word token.
pub struct AnchorEnd;

impl Step for AnchorEnd {
    fn step(&self, tokens: &[Token], cursor: usize, _source: &[char]) -> Option<isize> {
        if tokens
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, t)| !t.kind.is_whitespace())
            .map(|(i, _)| i)
            .next()
            == Some(cursor)
        {
            Some(0)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::{Expr, ExprExt, SequenceExpr};
    use crate::linting::tests::assert_suggestion_result;
    use crate::linting::{ExprLinter, Lint, Suggestion};
    use crate::{Document, Span, Token, TokenStringExt};

    use super::AnchorEnd;

    #[test]
    fn matches_period() {
        let document = Document::new_markdown_default_curated("This is a test.");
        let matches: Vec<_> = AnchorEnd.iter_matches_in_doc(&document).collect();

        assert_eq!(matches, vec![Span::new(7, 7)])
    }

    #[test]
    fn does_not_match_empty() {
        let document = Document::new_markdown_default_curated("");
        let matches: Vec<_> = AnchorEnd.iter_matches_in_doc(&document).collect();

        assert_eq!(matches, vec![])
    }

    pub struct End {
        expr: Box<dyn Expr>,
    }

    impl Default for End {
        fn default() -> Self {
            Self {
                expr: Box::new(SequenceExpr::default().then_any_word().then(AnchorEnd)), // Only replaces the very first word
                // expr: Box::new(SequenceExpr::default().then(AnchorEnd).then_any_word()), // Doesn't match anything!
                // expr: Box::new(SequenceExpr::default().then(AnchorEnd)), // Doesn't match anything!
            }
        }
    }

    impl ExprLinter for End {
        fn expr(&self) -> &dyn Expr {
            self.expr.as_ref()
        }

        fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
            // eprintln!("❤️ AnchorEnd: {:?}", toks.span()?.get_content_string(_src));
            Some(Lint {
                span: toks[0].span,
                suggestions: vec![Suggestion::ReplaceWith("END".chars().collect())],
                ..Default::default()
            })
        }

        fn description(&self) -> &str {
            "Testing `AnchorEnd`."
        }
    }

    #[test]
    fn flags_single_token() {
        assert_suggestion_result(
            "Hello, World! One two three four five.",
            End::default(),
            "Hello, END! One two three four END.",
        );
    }
}
