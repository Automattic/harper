use crate::linting::{Lint, Linter};
use crate::Document;

use super::matcher::Matcher;

pub struct MatcherSpelling {
    matcher: Matcher,
}

impl Default for MatcherSpelling {
    fn default() -> Self {
        Self {
            matcher: Matcher::new(true),
        }
    }
}

impl Linter for MatcherSpelling {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        self.matcher.lint(document)
    }

    fn description(&self) -> &'static str {
        "A collection of curated spelling rules: \"Expected ... instead\". A catch-all that will be removed in the future."
    }
}
