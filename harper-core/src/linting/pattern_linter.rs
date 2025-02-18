use super::{Lint, Linter};
use crate::patterns::Pattern;
use crate::{Token, TokenStringExt};

/// A trait that searches for [`Pattern`]s in [`Document`](crate::Document)s.
///
/// Makes use of [`TokenStringExt::iter_chunks`] to avoid matching across sentence or clause
/// boundaries.
#[cfg(not(feature = "concurrent"))]
pub trait PatternLinter {
    /// A simple getter for the pattern to be searched for.
    fn pattern(&self) -> &dyn Pattern;
    /// If any portions of a [`Document`](crate::Document) match [`Self::pattern`], they are passed through [`PatternLinter::match_to_lint`] to be
    /// transformed into a [`Lint`] for editor consumption.
    ///
    /// This function may return `None` to elect _not_ to produce a lint.
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint>;
    /// A user-facing description of what kinds of grammatical errors this rule looks for.
    /// It is usually shown in settings menus.
    fn description(&self) -> &str;
}

/// A trait that searches for [`Pattern`]s in [`Document`](crate::Document)s.
///
/// Makes use of [`TokenStringExt::iter_chunks`] to avoid matching across sentence or clause
/// boundaries.
#[cfg(feature = "concurrent")]
pub trait PatternLinter: Send + Sync {
    /// A simple getter for the pattern to be searched for.
    fn pattern(&self) -> &dyn Pattern;
    /// If any portions of a [`Document`](crate::Document) match [`Self::pattern`], they are passed through [`PatternLinter::match_to_lint`] to be
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
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let source = document.get_source();

        for chunk in document.iter_chunks() {
            let mut tok_cursor = 0;

            loop {
                if tok_cursor >= chunk.len() {
                    break;
                }

                let match_len = self.pattern().matches(&chunk[tok_cursor..], source);

                if match_len != 0 {
                    let lint =
                        self.match_to_lint(&chunk[tok_cursor..tok_cursor + match_len], source);

                    lints.extend(lint);
                    tok_cursor += match_len;
                } else {
                    tok_cursor += 1;
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        self.description()
    }
}
