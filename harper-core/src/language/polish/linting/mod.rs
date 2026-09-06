//! Polish linting support.

pub mod polish_spell_check;
pub mod weir_rules;

pub use polish_spell_check::PolishSpellCheck;

use crate::language::polish::dialects::PolishDialect;
use crate::linting::LintGroup;
use crate::spell::Dictionary;
use std::sync::Arc;

/// Create a curated Polish lint group.
pub fn new_curated_polish(
    _dialect: PolishDialect,
    _dictionary: Arc<impl Dictionary + 'static>,
) -> LintGroup {
    // For now, return an empty lint group - this will be populated with actual rules
    LintGroup::empty()
}

/// Get the Weir rules for Polish.
pub fn weir_rules() -> LintGroup {
    weir_rules::lint_group()
}
