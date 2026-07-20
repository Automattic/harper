//! Language module trait and implementations.
//!
//! This module provides the `LanguageModule` trait that each language must implement
//! to provide Harper with language-specific functionality.

use std::fmt::Debug;
use std::sync::Arc;

use crate::Token;
use crate::language::dialects::dialect_trait::Dialect;
use crate::language::languages::Language;
use crate::lexing::FoundToken;
use crate::linting::LintGroup;
use crate::parsers::Parser;
use crate::spell::{Dictionary, FstDictionary};

/// Core trait for language detectors.
///
/// This trait is used by the LanguageModule trait and implemented by each language's detector.
pub trait LanguageDetector: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn detect(&self, toks: &[Token], source: &[char], dict: &FstDictionary) -> Option<Language>;
    fn confidence(&self) -> f64;
}

/// Trait that each language module must implement.
///
/// This trait synthesizes all implementation that must be made for each language,
/// providing a standardized interface for Harper's core functionality.
///
/// # Implementation Notes
///
/// - Each language should create a struct that implements this trait
/// - The struct should delegate to the language's existing implementation
/// - For English, use an adapter pattern to point to master's existing files
/// - Associated types allow each language to use its own dialect and detector types
pub trait LanguageModule: 'static {
    /// Language variant enum
    type Dialect: Clone + Copy + Debug + PartialEq + Eq + Send + Sync + 'static + Dialect;

    /// Language identification implementation
    type Detector: LanguageDetector + 'static;

    /// Default dialect for this language
    fn default_dialect() -> Self::Dialect;

    /// Language identification detector instance
    fn detector() -> Self::Detector;

    /// Text tokenization implementation (low-level lexer)
    fn lex_token(source: &[char]) -> FoundToken;

    /// Plain text parser for this language
    fn plain_parser() -> impl Parser + 'static;

    /// Access to the language's spell-checking dictionary
    fn dictionary() -> Arc<FstDictionary>;

    /// All language-specific Rust linting rules
    fn rust_lint_group(dictionary: Arc<impl Dictionary + 'static>) -> LintGroup;

    /// All language-specific Weir rule linters
    fn weir_lint_group() -> LintGroup;

    /// Create a complete curated lint group for this language with a custom dictionary
    fn curated_lint_group(
        dialect: Self::Dialect,
        dictionary: Arc<impl Dictionary + 'static>,
    ) -> LintGroup;

    /// Serializes dialect flags for this language to JSON.
    /// Each language provides its own serialization logic for dialect flags.
    fn serialize_dialect_flags<S>(
        flags: &<Self::Dialect as Dialect>::Flags,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;

    /// Deserializes dialect flags for this language from JSON.
    /// Each language provides its own deserialization logic for dialect flags.
    fn deserialize_dialect_flags<'de, D>(
        deserializer: D,
    ) -> Result<<Self::Dialect as Dialect>::Flags, D::Error>
    where
        D: serde::Deserializer<'de>;
}
