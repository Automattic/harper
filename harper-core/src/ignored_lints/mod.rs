mod lint_context;

use hashbrown::HashSet;
pub use lint_context::LintContext;
use serde::{Deserialize, Serialize};

use crate::{Document, linting::Lint};

/// A structure that keeps track of lints that have been ignored by users.
///
/// To use this structure, apply [`Self::remove_ignored`] on the output of a
/// [`Linter`](crate::linting::Linter).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IgnoredLints {
    context_hashes: HashSet<u64>,
}

impl IgnoredLints {
    pub fn new() -> Self {
        Self::default()
    }

    /// Move entries from another instance to this one.
    pub fn append(&mut self, other: Self) {
        self.context_hashes.extend(other.context_hashes)
    }

    /// Add a lint to the list.
    pub fn ignore_lint(&mut self, lint: &Lint, document: &Document) {
        let context = LintContext::from_lint(lint, document);
        let context_hash = context.default_hash();

        self.ignore_hash(context_hash);
    }

    /// Add a context hash to the list of ignored lints.
    pub fn ignore_hash(&mut self, hash: u64) {
        self.context_hashes.insert(hash);
    }

    pub fn is_ignored(&self, lint: &Lint, document: &Document) -> bool {
        let context = LintContext::from_lint(lint, document);
        let hash = context.default_hash();

        self.context_hashes.contains(&hash)
    }

    /// Remove ignored Lints from a [`Vec`].
    pub fn remove_ignored(&self, lints: &mut Vec<Lint>, document: &Document) {
        if self.context_hashes.is_empty() {
            return;
        }

        lints.retain(|lint| !self.is_ignored(lint, document));
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use super::IgnoredLints;
    use crate::spell::FstDictionary;
    use crate::{
        Dialect, Document,
        linting::{LintGroup, Linter},
    };

    #[quickcheck]
    fn can_ignore_all(text: String) -> bool {
        let document = Document::new_markdown_default_curated(&text);

        let mut lints =
            LintGroup::new_curated(FstDictionary::curated(), Dialect::American).lint(&document);

        let mut ignored = IgnoredLints::new();

        for lint in &lints {
            ignored.ignore_lint(lint, &document);
        }

        ignored.remove_ignored(&mut lints, &document);
        lints.is_empty()
    }

    #[quickcheck]
    fn can_ignore_first(text: String) -> TestResult {
        let document = Document::new_markdown_default_curated(&text);

        let mut lints =
            LintGroup::new_curated(FstDictionary::curated(), Dialect::American).lint(&document);

        let Some(first) = lints.first().cloned() else {
            return TestResult::discard();
        };

        let mut ignored = IgnoredLints::new();
        ignored.ignore_lint(&first, &document);

        ignored.remove_ignored(&mut lints, &document);

        TestResult::from_bool(!lints.contains(&first))
    }

    // Check that ignoring the nth lint found in source text actually removes it (and no others).
    fn assert_ignore_lint_reduction(source: &str, nth_lint: usize) {
        let document = Document::new_markdown_default_curated(source);

        let mut lints =
            LintGroup::new_curated(FstDictionary::curated(), Dialect::American).lint(&document);

        let nth = lints.get(nth_lint).cloned().unwrap_or_else(|| {
            panic!("If ignoring the lint at {nth_lint}, make sure there are enough problems.")
        });

        let mut ignored = IgnoredLints::new();
        ignored.ignore_lint(&nth, &document);

        let prev_count = lints.len();

        ignored.remove_ignored(&mut lints, &document);

        assert_eq!(prev_count, lints.len() + 1);
        assert!(!lints.contains(&nth));
    }

    #[test]
    fn an_a() {
        let source = "There is an problem in this text. Here is an second one.";

        assert_ignore_lint_reduction(source, 0);
        assert_ignore_lint_reduction(source, 1);
    }

    #[test]
    fn spelling() {
        let source = "There is a problm in this text. Here is a scond one.";

        assert_ignore_lint_reduction(source, 0);
        assert_ignore_lint_reduction(source, 1);
    }
}
