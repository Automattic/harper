use crate::expr::{Expr, ExprExt};
use blanket::blanket;

use crate::{Document, LSend, Token, TokenStringExt};

use super::{Lint, Linter};

#[derive(Clone, Copy)]
pub enum Unit {
    /// Default: Process text in chunks (clauses between commas)
    Chunk,
    /// Process text in full sentences
    Sentence,
}

/// A trait that searches for tokens that fulfil [`Expr`]s in a [`Document`].
///
/// Makes use of [`TokenStringExt::iter_chunks`] by default, or [`TokenStringExt::iter_sentences`] to process either
/// a chunk (clause) or a sentence at a time.
#[blanket(derive(Box))]
pub trait ExprLinter: LSend {
    /// A simple getter for the expression you want Harper to search for.
    fn expr(&self) -> &dyn Expr;
    /// If any portions of a [`Document`] match [`Self::expr`], they are passed through [`ExprLinter::match_to_lint`] to be
    /// transformed into a [`Lint`] for editor consumption.
    ///
    /// This function may return `None` to elect _not_ to produce a lint.
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint>;
    /// A user-facing description of what kinds of grammatical errors this rule looks for.
    /// It is usually shown in settings menus.
    fn description(&self) -> &str;
    /// The unit of analysis for this linter.
    /// Use [`Unit::Sentence`] if you need commas to be included in the analysis.
    fn get_unit(&self) -> Unit {
        Unit::Chunk
    }
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

impl<L> Linter for L
where
    L: ExprLinter,
{
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let source = document.get_source();

        let iter: Box<dyn Iterator<Item = &[Token]>> = match self.get_unit() {
            Unit::Chunk => Box::new(document.iter_chunks()),
            Unit::Sentence => Box::new(document.iter_sentences()),
        };

        for chunk in iter {
            lints.extend(run_on_chunk(self, chunk, source));
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
            linter.match_to_lint(&unit[match_span.start..match_span.end], source)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::SequenceExpr;
    use crate::linting::Suggestion;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    pub struct FooBar {
        expr: Box<dyn Expr>,
        unit: Unit,
    }

    impl FooBar {
        pub fn new(unit: Unit) -> Self {
            Self {
                expr: Box::new(SequenceExpr::fixed_phrase("foo, bar")),
                unit,
            }
        }
    }

    impl Default for FooBar {
        fn default() -> Self {
            Self::new(Unit::Chunk)
        }
    }

    impl ExprLinter for FooBar {
        fn expr(&self) -> &dyn Expr {
            self.expr.as_ref()
        }

        fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
            Some(Lint {
                span: toks.span()?,
                suggestions: vec![Suggestion::ReplaceWith("foo and bar".chars().collect())],
                ..Default::default()
            })
        }

        fn description(&self) -> &str {
            "A test"
        }

        fn get_unit(&self) -> Unit {
            self.unit
        }
    }

    #[test]
    fn cant_match_a_phrase_containing_commas() {
        assert_no_lints("one, two, foo, bar, three, four", FooBar::default());
    }

    #[test]
    fn matches_a_phrase_containing_commas() {
        assert_suggestion_result(
            "one, two, foo, bar, three, four",
            FooBar::new(Unit::Sentence),
            "one, two, foo and bar, three, four",
        );
    }
}
