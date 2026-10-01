//! Polish linting support.

pub mod polish_spell_check;
pub mod weir_rules;

pub use polish_spell_check::PolishSpellCheck;

use crate::language::languages::Language;
use crate::language::polish::dialects::PolishDialect;
use crate::linting::LintGroup;
use crate::spell::Dictionary;
use std::sync::Arc;

/// Create a curated Polish lint group.
pub fn new_curated_polish(
    dialect: PolishDialect,
    dictionary: Arc<impl Dictionary + 'static>,
) -> LintGroup {
    use crate::language::module::LanguageModule;
    use crate::language::polish::module::PolishModule;
    use crate::language::registry::weir_rules_lint_group;

    let language = Language::Polish(dialect);

    let mut group = LintGroup::empty();
    group.merge_from(weir_rules_lint_group(language));
    group.merge_from(PolishModule::rust_lint_group(dictionary));
    group.set_all_rules_to(Some(true));

    group
}

/// Get the Weir rules for Polish.
pub fn weir_rules() -> LintGroup {
    weir_rules::lint_group()
}
