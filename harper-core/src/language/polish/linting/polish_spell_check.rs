use std::sync::Arc;

use crate::Document;
use crate::language::polish::dialects::PolishDialect;
use crate::linting::{Lint, Linter};
use crate::spell::Dictionary;

/// Polish spell check linter.
pub struct PolishSpellCheck {
    dictionary: Arc<dyn Dictionary + 'static>,
    dialect: PolishDialect,
}

impl PolishSpellCheck {
    /// Create a new Polish spell check linter.
    pub fn new(dictionary: Arc<impl Dictionary + 'static>, dialect: PolishDialect) -> Self {
        Self {
            dictionary,
            dialect,
        }
    }
}

impl Linter for PolishSpellCheck {
    fn lint(&mut self, _document: &Document) -> Vec<Lint> {
        // For now, return empty - this will be implemented with actual spell checking
        Vec::new()
    }

    fn description(&self) -> &str {
        "Checks for Polish spelling errors"
    }
}
