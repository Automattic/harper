//! German linting rules and checkers.

pub mod german_filler_words;
pub mod german_noun_capitalization;
pub mod german_sentence_capitalization;
pub mod german_spell_check;
pub mod weir_rules;

use crate::language::german::dialects::GermanDialect;
use crate::language::german::spell::combined_german_dictionary;
use crate::language::languages::Language;
use crate::linting::LintGroup;

/// Create a new curated lint group for German language.
pub fn new_curated_german(dialect: GermanDialect) -> LintGroup {
    use crate::language::german::module::GermanModule;
    use crate::language::module::LanguageModule;
    use crate::language::registry::weir_rules_lint_group;

    let dictionary = combined_german_dictionary();
    let language = Language::German(dialect);

    let mut group = LintGroup::empty();
    group.merge_from(weir_rules_lint_group(language));
    group.merge_from(GermanModule::rust_lint_group(dictionary));
    group.set_all_rules_to(Some(true));

    group
}
