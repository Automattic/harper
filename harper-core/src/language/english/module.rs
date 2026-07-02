//! English language module implementation of LanguageModule trait.
//!
//! This implementation points to the existing English functionality in master
//! to allow easy merging of upstream changes.

use std::sync::Arc;

use crate::language::english::dialects::EnglishDialect;
use crate::language::english::language_detection::EnglishDetector;
use crate::lexing::{FoundToken, lex_english_token};
use crate::linting::{LintGroup, weir_rules};
use crate::parsers::{Parser, PlainEnglish};
use crate::spell::{Dictionary, FstDictionary};

use crate::language::module::LanguageModule;

/// English language module implementing the LanguageModule trait.
///
/// This delegates to the existing English functionality that exists in master.
pub struct EnglishModule;

impl LanguageModule for EnglishModule {
    type Dialect = EnglishDialect;
    type Detector = EnglishDetector;

    fn default_dialect() -> Self::Dialect {
        EnglishDialect::default()
    }

    fn detector() -> Self::Detector {
        EnglishDetector
    }

    fn lex_token(source: &[char]) -> FoundToken {
        lex_english_token(source)
    }

    fn plain_parser() -> impl Parser + 'static {
        PlainEnglish
    }

    fn dictionary() -> Arc<FstDictionary> {
        FstDictionary::curated()
    }

    fn rust_lint_group(_dictionary: Arc<impl Dictionary + 'static>) -> LintGroup {
        // English doesn't have language-specific Rust linters beyond the main group
        // All English linters are added directly to LintGroup::new_curated()
        LintGroup::empty()
    }

    fn weir_lint_group() -> LintGroup {
        weir_rules::lint_group()
    }

    fn curated_lint_group(dialect: Self::Dialect) -> LintGroup {
        LintGroup::new_curated(Self::dictionary(), dialect)
    }
}
