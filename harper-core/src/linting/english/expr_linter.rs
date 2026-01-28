use crate::expr::{Expr, ExprExt};
use blanket::blanket;

use crate::{Document, LSend, Token, TokenStringExt};

use super::{Lint, Linter};

/// A trait that searches for tokens that fulfil [`Expr`]s in a [`Document`].
///
/// Makes use of [`TokenStringExt::iter_chunks`] to avoid matching across sentence or clause
/// boundaries.
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

        for chunk in document.iter_chunks() {
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
    chunk: &'a [Token],
    source: &'a [char],
) -> impl Iterator<Item = Lint> + 'a {
    linter
        .expr()
        .iter_matches(chunk, source)
        .filter_map(|match_span| {
            linter.match_to_lint(&chunk[match_span.start..match_span.end], source)
        })
}
