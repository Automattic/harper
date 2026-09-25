//! Portuguese language module implementation of LanguageModule trait.

use std::sync::Arc;

use crate::language::portuguese::dialects::PortugueseDialect;
use crate::language::portuguese::language_detection::PortugueseDetector;
use crate::language::portuguese::linting::{new_curated_portuguese, weir_rules};
use crate::language::portuguese::parsers::PlainPortuguese;
use crate::language::portuguese::spell::portuguese_dictionary;
use crate::linting::LintGroup;
use crate::parsers::Parser;
use crate::spell::Dictionary;

use crate::language::module::LanguageModule;

/// Portuguese language module implementing the LanguageModule trait.
pub struct PortugueseModule;

impl LanguageModule for PortugueseModule {
    type Dialect = PortugueseDialect;
    type Detector = PortugueseDetector;

    fn default_dialect() -> Self::Dialect {
        PortugueseDialect::default()
    }

    fn detector() -> Self::Detector {
        PortugueseDetector
    }

    fn plain_parser() -> impl Parser + 'static {
        PlainPortuguese
    }

    fn dictionary() -> Arc<dyn Dictionary> {
        portuguese_dictionary()
    }

    fn rust_lint_group(dictionary: Arc<impl Dictionary + 'static>) -> LintGroup {
        use crate::language::portuguese::linting::portuguese_spell_check::PortugueseSpellCheck;

        let mut group = LintGroup::empty();
        group.add(
            "portuguese_spell_check",
            PortugueseSpellCheck::new(dictionary, PortugueseDialect::default()),
        );
        group
    }

    fn weir_lint_group() -> LintGroup {
        weir_rules::lint_group()
    }

    fn curated_lint_group(
        dialect: Self::Dialect,
        dictionary: Arc<impl Dictionary + 'static>,
    ) -> LintGroup {
        new_curated_portuguese(dialect, dictionary)
    }
}
