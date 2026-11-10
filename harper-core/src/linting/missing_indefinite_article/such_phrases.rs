use super::phrase::{lint_phrases, previous};
use crate::{
    CharStringExt, Dialect, Document,
    linting::{Lint, Linter},
};

/// Places an indefinite article after `such`, rather than before the quantifier.
/// Unmodified phrases remain unchecked because some permit a bare use.
pub(super) struct SuchPhrases(pub Dialect);

impl Linter for SuchPhrases {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        lint_phrases(document, self.0, |tokens, phrase, source| {
            phrase.first.span != phrase.head.span
                && previous(tokens, phrase)
                    .is_some_and(|t| t.get_ch(source).eq_any_ignore_ascii_case_str(&["such"]))
        })
    }
    fn description(&self) -> &'static str {
        "Checks missing articles after `such`."
    }
}
