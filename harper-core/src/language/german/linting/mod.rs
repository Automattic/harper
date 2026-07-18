//! German linting rules and checkers.

pub mod german_filler_words;
pub mod german_noun_capitalization;
pub mod german_sentence_capitalization;
pub mod german_spell_check;
pub mod weir_rules;

use std::sync::Arc;

use crate::language::german::dialects::GermanDialect;
use crate::language::german::module::GermanModule;
use crate::language::languages::Language;
use crate::language::module::LanguageModule;
use crate::language::registry::weir_rules_lint_group;
use crate::linting::LintGroup;
use crate::spell::Dictionary;

/// Create a new curated lint group for German language with a custom dictionary.
pub fn new_curated_german(
    dialect: GermanDialect,
    dictionary: Arc<impl Dictionary + 'static>,
) -> LintGroup {
    let language = Language::German(dialect);

    let mut group = LintGroup::empty();
    group.merge_from(weir_rules_lint_group(language));
    group.merge_from(GermanModule::rust_lint_group(dictionary));
    group.set_all_rules_to(Some(true));

    group
}
