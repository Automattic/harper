use crate::expr::{Expr, ExprExt};
use blanket::blanket;

use crate::{Document, LSend, Token, TokenStringExt};

use super::{Lint, Linter};

pub trait DocumentIterator {
    type Unit;

    fn iter_units<'a>(document: &'a Document) -> Box<dyn Iterator<Item = &'a [Token]> + 'a>;
}

/// Process text in chunks (clauses between commas)
pub struct Chunk;
/// Process text in full sentences
pub struct Sentence;

impl DocumentIterator for Chunk {
    type Unit = Chunk;

    fn iter_units<'a>(document: &'a Document) -> Box<dyn Iterator<Item = &'a [Token]> + 'a> {
        Box::new(document.iter_chunks())
    }
}

impl DocumentIterator for Sentence {
    type Unit = Sentence;

    fn iter_units<'a>(document: &'a Document) -> Box<dyn Iterator<Item = &'a [Token]> + 'a> {
        Box::new(document.iter_sentences())
    }
}

/// A trait that searches for tokens that fulfil [`Expr`]s in a [`Document`].
///
/// Makes use of [`TokenStringExt::iter_chunks`] by default, or [`TokenStringExt::iter_sentences`] to process either
/// a chunk (clause) or a sentence at a time.
#[blanket(derive(Box))]
pub trait ExprLinter: LSend {
    type Unit: DocumentIterator;

    /// A simple getter for the expression you want Harper to search for.
    fn expr(&self) -> &dyn Expr;
    /// If any portions of a [`Document`] match [`Self::expr`], they are passed through [`ExprLinter::match_to_lint`]
    /// or [`ExprLinter::match_to_lint_with_context`] to be transformed into a [`Lint`] for editor consumption.
    ///
    /// Transform matched tokens into a [`Lint`] for editor consumption.
    ///
    /// This is the simple version that only sees the matched tokens. For context-aware linting,
    /// implement `match_to_lint_with_context` instead.
    ///
    /// Return `None` to skip producing a lint for this match.
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        self.match_to_lint_with_context(matched_tokens, source, None)
    }

    /// Transform matched tokens into a [`Lint`] with access to surrounding context.
    ///
    /// The context provides access to tokens before and after the match. When implementing
    /// this method, you can call `self.match_to_lint()` as a fallback if the context isn't needed.
    ///
    /// Return `None` to skip producing a lint for this match.
    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        _context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // Default implementation falls back to the simple version
        self.match_to_lint(matched_tokens, source)
    }
    /// A user-facing description of what kinds of grammatical errors this rule looks for.
    /// It is usually shown in settings menus.
    fn description(&self) -> &str;
}

/// Helper function to find the only occurrence of a token matching a predicate
///
/// Returns `Some(token)` if exactly one token matches the predicate, `None` otherwise.
/// TODO: This can be used in the [`ThenThan`] linter when #1819 is merged.
pub fn find_the_only_token_matching<'a, F>(
    tokens: &'a [Token],
    source: &[char],
    predicate: F,
) -> Option<&'a Token>
where
    F: Fn(&Token, &[char]) -> bool,
{
    let mut matches = tokens.iter().filter(|&tok| predicate(tok, source));
    match (matches.next(), matches.next()) {
        (Some(tok), None) => Some(tok),
        _ => None,
    }
}

impl<L, U> Linter for L
where
    L: ExprLinter<Unit = U>,
    U: DocumentIterator,
{
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let source = document.get_source();

        for unit in U::iter_units(document) {
            lints.extend(run_on_chunk(self, unit, source));
        }

        lints
    }

    fn description(&self) -> &str {
        self.description()
    }
}

pub fn run_on_chunk<'a>(
    linter: &'a impl ExprLinter,
    unit: &'a [Token],
    source: &'a [char],
) -> impl Iterator<Item = Lint> + 'a {
    linter
        .expr()
        .iter_matches(unit, source)
        .filter_map(|match_span| {
            linter.match_to_lint_with_context(
                &unit[match_span.start..match_span.end],
                source,
                Some((&unit[..match_span.start], &unit[match_span.end..])),
            )
        })
}

#[cfg(test)]
mod tests_context {
    use crate::expr::{Expr, FixedPhrase};
    use crate::linting::expr_linter::{Chunk, Sentence};
    use crate::linting::tests::assert_suggestion_result;
    use crate::linting::{ExprLinter, Suggestion};
    use crate::token_string_ext::TokenStringExt;
    use crate::{Lint, Token};

    pub struct TestSimpleLinter {
        expr: Box<dyn Expr>,
    }

    impl Default for TestSimpleLinter {
        fn default() -> Self {
            Self {
                expr: Box::new(FixedPhrase::from_phrase("two")),
            }
        }
    }

    impl ExprLinter for TestSimpleLinter {
        type Unit = Chunk;

        fn expr(&self) -> &dyn Expr {
            &*self.expr
        }

        fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
            Some(Lint {
                span: toks.span()?,
                message: "simple".to_string(),
                suggestions: vec![Suggestion::ReplaceWith(vec!['2'])],
                ..Default::default()
            })
        }

        fn description(&self) -> &str {
            "test linter"
        }
    }

    pub struct TestContextLinter {
        expr: Box<dyn Expr>,
    }

    impl Default for TestContextLinter {
        fn default() -> Self {
            Self {
                expr: Box::new(FixedPhrase::from_phrase("two")),
            }
        }
    }

    impl ExprLinter for TestContextLinter {
        type Unit = Chunk;

        fn expr(&self) -> &dyn Expr {
            &*self.expr
        }

        fn match_to_lint_with_context(
            &self,
            toks: &[Token],
            src: &[char],
            context: Option<(&[Token], &[Token])>,
        ) -> Option<Lint> {
            if let Some((before, after)) = context {
                let before = before.span()?.get_content_string(src);
                let after = after.span()?.get_content_string(src);

                let (message, suggestions) = if before.eq_ignore_ascii_case("one ")
                    && after.eq_ignore_ascii_case(" three")
                {
                    (
                        "ascending".to_string(),
                        vec![Suggestion::ReplaceWith(vec!['>'])],
                    )
                } else if before.eq_ignore_ascii_case("three ")
                    && after.eq_ignore_ascii_case(" one")
                {
                    (
                        "descending".to_string(),
                        vec![Suggestion::ReplaceWith(vec!['<'])],
                    )
                } else {
                    (
                        "dunno".to_string(),
                        vec![Suggestion::ReplaceWith(vec!['?'])],
                    )
                };

                return Some(Lint {
                    span: toks.span()?,
                    message,
                    suggestions,
                    ..Default::default()
                });
            } else {
                None
            }
        }

        fn description(&self) -> &str {
            "context linter"
        }
    }

    pub struct TestSentenceLinter {
        expr: Box<dyn Expr>,
    }

    impl Default for TestSentenceLinter {
        fn default() -> Self {
            Self {
                expr: Box::new(FixedPhrase::from_phrase("two, two")),
            }
        }
    }

    impl ExprLinter for TestSentenceLinter {
        type Unit = Sentence;

        fn expr(&self) -> &dyn Expr {
            self.expr.as_ref()
        }

        fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
            Some(Lint {
                span: toks.span()?,
                message: "sentence".to_string(),
                suggestions: vec![Suggestion::ReplaceWith(vec!['2', '&', '2'])],
                ..Default::default()
            })
        }

        fn description(&self) -> &str {
            "sentence linter"
        }
    }

    #[test]
    fn simple_test_123() {
        assert_suggestion_result("one two three", TestSimpleLinter::default(), "one 2 three");
    }

    #[test]
    fn context_test_123() {
        assert_suggestion_result("one two three", TestContextLinter::default(), "one > three");
    }

    #[test]
    fn context_test_321() {
        assert_suggestion_result("three two one", TestContextLinter::default(), "three < one");
    }

    #[test]
    fn sentence_test_123() {
        assert_suggestion_result(
            "one, two, two, three",
            TestSentenceLinter::default(),
            "one, 2&2, three",
        );
    }
}
