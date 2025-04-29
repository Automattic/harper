use blanket::blanket;

use crate::{
    Document, LSend, Token, TokenStringExt,
    patterns::{Pattern, PatternExt},
};

use super::{Lint, Linter};

/// A trait that searches for [`Pattern`]s in [`Document`]s.
///
/// Makes use of [`TokenStringExt::iter_chunks`] to avoid matching across sentence or clause
/// boundaries.
#[blanket(derive(Box))]
pub trait PatternLinter: LSend {
    /// A simple getter for the pattern to be searched for.
    fn pattern(&self) -> &dyn Pattern;
    /// If any portions of a [`Document`] match [`Self::pattern`], they are passed through [`PatternLinter::match_to_lint`] to be
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
    L: PatternLinter,
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

pub fn run_on_chunk(linter: &impl PatternLinter, chunk: &[Token], source: &[char]) -> Vec<Lint> {
    let mut lints = Vec::new();

    for match_span in linter.pattern().iter_matches(chunk, source) {
        let lint = linter.match_to_lint(&chunk[match_span.start..match_span.end], source);
        lints.extend(lint);
    }

    lints
}
