//! Polish language module implementation of LanguageModule trait.

use std::sync::Arc;

use crate::language::polish::dialects::PolishDialect;
use crate::language::polish::language_detection::PolishDetector;
use crate::language::polish::linting::{new_curated_polish, weir_rules};
use crate::language::polish::parsers::PlainPolish;
use crate::language::polish::spell::polish_dictionary;
use crate::linting::LintGroup;
use crate::parsers::Parser;
use crate::spell::Dictionary;

use crate::language::module::LanguageModule;

/// Polish language module implementing the LanguageModule trait.
pub struct PolishModule;

impl LanguageModule for PolishModule {
    type Dialect = PolishDialect;
    type Detector = PolishDetector;

    fn default_dialect() -> Self::Dialect {
        PolishDialect::default()
    }

    fn detector() -> Self::Detector {
        PolishDetector
    }

    fn plain_parser() -> impl Parser + 'static {
        PlainPolish
    }

    fn dictionary() -> Arc<dyn Dictionary> {
        polish_dictionary()
    }

    fn rust_lint_group(dictionary: Arc<impl Dictionary + 'static>) -> LintGroup {
        use crate::language::polish::linting::polish_spell_check::PolishSpellCheck;

        let mut group = LintGroup::empty();
        group.add(
            "PolishSpellCheck",
            PolishSpellCheck::new(dictionary.clone(), PolishDialect::default()),
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
        new_curated_polish(dialect, dictionary)
    }
}
