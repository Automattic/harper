//! Portuguese language module implementation of LanguageModule trait.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::language::dialects::dialect_trait::Dialect;
use crate::language::portuguese::dialects::{PortugueseDialect, PortugueseDialectFlags};
use crate::language::portuguese::language_detection::PortugueseDetector;
use crate::language::portuguese::lexing::lex_portuguese_token;
use crate::language::portuguese::linting::{new_curated_portuguese, weir_rules};
use crate::language::portuguese::parsers::PlainPortuguese;
use crate::language::portuguese::spell::portuguese_dictionary;
use crate::lexing::FoundToken;
use crate::linting::LintGroup;
use crate::parsers::Parser;
use crate::spell::{Dictionary, FstDictionary};

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

    fn lex_token(source: &[char]) -> FoundToken {
        lex_portuguese_token(source)
    }

    fn plain_parser() -> impl Parser + 'static {
        PlainPortuguese
    }

    fn dictionary() -> Arc<FstDictionary> {
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

    fn serialize_dialect_flags<S>(
        flags: &<Self::Dialect as Dialect>::Flags,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        flags.serialize(serializer)
    }

    fn deserialize_dialect_flags<'de, D>(
        deserializer: D,
    ) -> Result<<Self::Dialect as Dialect>::Flags, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        PortugueseDialectFlags::deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spell::FstDictionary;

    #[test]
    fn test_rust_lint_group_contains_spell_check() {
        let dict = Arc::new(FstDictionary::curated()); // Use a dummy dictionary for testing
        let group = PortugueseModule::rust_lint_group(dict);

        // Check if the spell check linter was added
        assert!(
            group.contains_key("portuguese_spell_check"),
            "rust_lint_group should contain portuguese_spell_check linter"
        );
    }
}
