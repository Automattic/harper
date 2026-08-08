//! German language module implementation of LanguageModule trait.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::language::dialects::dialect_trait::Dialect;
use crate::language::german::dialects::{GermanDialect, GermanDialectFlags};
use crate::language::german::language_detection::GermanDetector;
use crate::language::german::lexing::lex_german_token;
use crate::language::german::linting::{new_curated_german, weir_rules};
use crate::language::german::parsers::PlainGerman;

use crate::lexing::FoundToken;
use crate::linting::LintGroup;
use crate::parsers::Parser;
use crate::spell::{Dictionary, FstDictionary};

use crate::language::module::LanguageModule;

/// German language module implementing the LanguageModule trait.
pub struct GermanModule;

impl LanguageModule for GermanModule {
    type Dialect = GermanDialect;
    type Detector = GermanDetector;

    fn default_dialect() -> Self::Dialect {
        GermanDialect::default()
    }

    fn detector() -> Self::Detector {
        GermanDetector
    }

    fn lex_token(source: &[char]) -> FoundToken {
        lex_german_token(source)
    }

    fn plain_parser() -> impl Parser + 'static {
        PlainGerman
    }

    fn dictionary() -> Arc<FstDictionary> {
        // Use curated dictionary for German which includes all word forms and metadata
        // This provides comprehensive German language support with proper annotations
        use crate::language::german::spell::curated_german_dictionary;
        curated_german_dictionary()
    }

    fn rust_lint_group(dictionary: Arc<impl Dictionary + 'static>) -> LintGroup {
        use crate::language::german::linting::{
            german_adjective_agreement::GermanAdjectiveAgreement,
            german_case_usage::GermanCaseUsage, german_filler_words::GermanFillerWords,
            german_noun_capitalization::GermanNounCapitalization,
            german_noun_declension::GermanNounDeclension,
            german_pronoun_agreement::GermanPronounAgreement,
            german_sentence_capitalization::GermanSentenceCapitalization,
            german_spell_check::GermanSpellCheck,
            german_subject_verb_agreement::GermanSubjectVerbAgreement,
        };

        let mut group = LintGroup::empty();
        group.add(
            "GermanSpellCheck",
            GermanSpellCheck::new(dictionary.clone()),
        );
        group.add(
            "GermanAdjectiveAgreement",
            GermanAdjectiveAgreement::new(dictionary.clone()),
        );
        group.add("GermanCaseUsage", GermanCaseUsage::new(dictionary.clone()));
        group.add(
            "GermanNounDeclension",
            GermanNounDeclension::new(dictionary.clone()),
        );
        group.add(
            "GermanPronounAgreement",
            GermanPronounAgreement::new(dictionary.clone()),
        );
        group.add(
            "GermanSubjectVerbAgreement",
            GermanSubjectVerbAgreement::new(dictionary.clone()),
        );
        group.add(
            "GermanNounCapitalization",
            GermanNounCapitalization::new(dictionary.clone()),
        );
        group.add(
            "GermanSentenceCapitalization",
            GermanSentenceCapitalization::new(dictionary.clone()),
        );
        group.add("GermanFillerWords", GermanFillerWords::default());
        group
    }

    fn weir_lint_group() -> LintGroup {
        weir_rules::lint_group()
    }

    fn curated_lint_group(
        dialect: Self::Dialect,
        dictionary: Arc<impl Dictionary + 'static>,
    ) -> LintGroup {
        new_curated_german(dialect, dictionary)
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
        GermanDialectFlags::deserialize(deserializer)
    }
}
