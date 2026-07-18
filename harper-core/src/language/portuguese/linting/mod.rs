//! Portuguese linting rules and checkers.

pub mod portuguese_spell_check;
pub mod weir_rules;

use std::sync::Arc;

use crate::language::languages::Language;
use crate::language::portuguese::dialects::PortugueseDialect;
use crate::linting::LintGroup;
use crate::spell::Dictionary;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_curated_portuguese_contains_spell_check() {
        use crate::language::portuguese::spell::portuguese_dictionary;
        let group = new_curated_portuguese(PortugueseDialect::Brazilian, portuguese_dictionary());

        // Check if the spell check linter was added
        assert!(
            group.contains_key("portuguese_spell_check"),
            "new_curated_portuguese should contain portuguese_spell_check linter"
        );
    }
}

/// Create a new curated lint group for Portuguese language with a custom dictionary.
pub fn new_curated_portuguese(dialect: PortugueseDialect, dictionary: Arc<impl Dictionary + 'static>) -> LintGroup {
    use crate::language::registry::{add_language_specific_linters, weir_rules_lint_group};

    let language = Language::Portuguese(dialect);

    let mut group = LintGroup::default(); // Use default() instead of empty()
    group.merge_from(weir_rules_lint_group(language));
    add_language_specific_linters(&mut group, language, dictionary);
    group.set_all_rules_to(Some(true)); // Explicitly enable all linters

    group
}
