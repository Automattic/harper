//! Slovak linting rules and checkers.

pub mod slovak_spell_check;
pub mod weir_rules;

use crate::language::languages::Language;
use crate::language::slovak::dialects::SlovakDialect;
use crate::language::slovak::spell::curated_slovak_dictionary;
use crate::linting::LintGroup;

/// Create a new curated lint group for Slovak language.
pub fn new_curated_slovak(dialect: SlovakDialect) -> LintGroup {
    use crate::language::module::LanguageModule;
    use crate::language::registry::weir_rules_lint_group;
    use crate::language::slovak::module::SlovakModule;

    let dictionary = curated_slovak_dictionary();
    let language = Language::Slovak(dialect);

    let mut group = LintGroup::empty();
    group.merge_from(weir_rules_lint_group(language));
    group.merge_from(SlovakModule::rust_lint_group(dictionary));
    group.set_all_rules_to(Some(true));

    group
}