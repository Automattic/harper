//! Slovak linting rules and checkers.

pub mod slovak_spell_check;
pub mod weir_rules;

use std::sync::Arc;

use crate::language::languages::Language;
use crate::language::slovak::dialects::SlovakDialect;
use crate::linting::LintGroup;
use crate::spell::Dictionary;

/// Create a new curated lint group for Slovak language with a custom dictionary.
pub fn new_curated_slovak(
    dialect: SlovakDialect,
    dictionary: Arc<impl Dictionary + 'static>,
) -> LintGroup {
    use crate::language::module::LanguageModule;
    use crate::language::registry::weir_rules_lint_group;
    use crate::language::slovak::module::SlovakModule;

    let language = Language::Slovak(dialect);

    let mut group = LintGroup::empty();
    group.merge_from(weir_rules_lint_group(language));
    group.merge_from(SlovakModule::rust_lint_group(dictionary));
    group.set_all_rules_to(Some(true));

    group
}
