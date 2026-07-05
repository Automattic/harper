//! Slovak language module implementation of LanguageModule trait.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::language::dialects::dialect_trait::Dialect;
use crate::language::slovak::dialects::{SlovakDialect, SlovakDialectFlags};
use crate::language::slovak::language_detection::SlovakDetector;
use crate::language::slovak::lexing::lex_slovak_token;
use crate::language::slovak::linting::{new_curated_slovak, weir_rules};
use crate::language::slovak::parsers::PlainSlovak;
use crate::language::slovak::spell::curated_slovak_dictionary;

use crate::lexing::FoundToken;
use crate::linting::LintGroup;
use crate::parsers::Parser;
use crate::spell::{Dictionary, FstDictionary};

use crate::language::module::LanguageModule;

/// Slovak language module implementing the LanguageModule trait.
pub struct SlovakModule;

impl LanguageModule for SlovakModule {
    type Dialect = SlovakDialect;
    type Detector = SlovakDetector;

    fn default_dialect() -> Self::Dialect {
        SlovakDialect::default()
    }

    fn detector() -> Self::Detector {
        SlovakDetector
    }

    fn lex_token(source: &[char]) -> FoundToken {
        lex_slovak_token(source)
    }

    fn plain_parser() -> impl Parser + 'static {
        PlainSlovak
    }

    fn dictionary() -> Arc<FstDictionary> {
        curated_slovak_dictionary()
    }

    fn rust_lint_group(dictionary: Arc<impl Dictionary + 'static>) -> LintGroup {
        use crate::language::slovak::dialects::SlovakDialect;
        use crate::language::slovak::linting::slovak_spell_check::SlovakSpellCheck;

        let mut group = LintGroup::empty();
        group.add(
            "SlovakSpellCheck",
            SlovakSpellCheck::new(dictionary.clone(), SlovakDialect::default()),
        );
        group
    }

    fn weir_lint_group() -> LintGroup {
        weir_rules::lint_group()
    }

    fn curated_lint_group(dialect: Self::Dialect) -> LintGroup {
        new_curated_slovak(dialect)
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
        SlovakDialectFlags::deserialize(deserializer)
    }
}
