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
