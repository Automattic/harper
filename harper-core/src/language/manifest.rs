//! Language manifest - central integration point for all Harper languages.
//!
//! This module serves as the single integration point for all language-specific features.
//! 
//! ## Language Structure
//! 
//! - **English** (default): Core language files are in `src/linting/english/` for Rust-based rules
//!   and `src/linting/weir_rules/` for Weir-based rules. English dialects are in `src/dialects/english.rs`.
//! - **German**: Language-specific files are in `src/language/german/` (linting, spell checking, etc.)
//!   with dialects in `src/language/german/dialects.rs`.
//! - **Portuguese**: Language-specific files are in `src/language/portuguese/` with dialects in `src/dialects/portuguese.rs`.
//!
//! To add a new language:
//! 1. Create the language module under `language/<name>/`
//! 2. Add imports for the new language below
//! 3. Add entries to the appropriate sections (DETECTORS, DICTIONARIES, etc.)

use std::sync::Arc;

use crate::languages::{Language, LanguageFamily};
use crate::spell::{Dictionary, FstDictionary};
use crate::{
    LintGroup,
    parsers::{Markdown, MarkdownOptions, OrgMode, Parser, PlainEnglish},
};

// ========== IMPORTS - Add new language imports here ==========

// German
use crate::language::german::linting::{
    german_filler_words::GermanFillerWords, german_noun_capitalization::GermanNounCapitalization,
    german_sentence_capitalization::GermanSentenceCapitalization,
    german_spell_check::GermanSpellCheck,
};
use crate::language::german::parsers::PlainGerman;
use crate::language::german::spell::german_dictionary;
use crate::language_detection::german::GermanDetector;

// Portuguese
use crate::language::portuguese::linting::portuguese_spell_check::PortugueseSpellCheck;
use crate::language::portuguese::parsers::PlainPortuguese;
use crate::language::portuguese::spell::portuguese_dictionary;
use crate::language_detection::portuguese::PortugueseDetector;

// English (built-in)
use crate::language_detection::english::EnglishDetector;

// ========== DETECTION ==========

use crate::language_detection::LanguageDetector;

/// All language detectors, sorted by confidence (highest to lowest).
///
/// To add a new detector:
/// 1. Import the detector type above
/// 2. Add an entry here with its confidence value
static DETECTORS: &[(&'static dyn LanguageDetector, f64)] = &[
    // German has highest confidence due to unique characters (ß, ä, ö, ü)
    (&GermanDetector as &dyn LanguageDetector, 0.95),
    // Portuguese has high confidence due to unique characters (ã, õ, ç)
    (&PortugueseDetector as &dyn LanguageDetector, 0.90),
    // English is the fallback with lower confidence
    (&EnglishDetector as &dyn LanguageDetector, 0.30),
];

/// Detect the language of the given source text.
pub fn detect_language(source: &str, dict: &FstDictionary, default_language: Language) -> Language {
    use crate::parsers::PlainEnglish;

    let source_chars: Vec<char> = source.chars().collect();
    let tokens = PlainEnglish.parse(&source_chars);

    if tokens.is_empty() {
        return default_language;
    }

    for (detector, _confidence) in DETECTORS {
        if let Some(language) = detector.detect(&tokens, &source_chars, dict, default_language) {
            return language;
        }
    }
    default_language
}

// ========== PROSE LANGUAGE ==========

/// Prose languages supported by Harper for text parsing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProseLanguage {
    English,
    German,
    Portuguese,
}

/// Convert a Harper Language to a ProseLanguage.
pub fn prose_language(language: &Language) -> ProseLanguage {
    match language {
        Language::English(_) => ProseLanguage::English,
        Language::German(_) => ProseLanguage::German,
        Language::Portuguese(_) => ProseLanguage::Portuguese,
    }
}

// ========== DICTIONARIES ==========

/// Get the dictionary for a language family.
pub fn dictionary_for_language(family: LanguageFamily) -> Arc<FstDictionary> {
    match family {
        LanguageFamily::English => FstDictionary::curated(),
        LanguageFamily::German => german_dictionary(),
        LanguageFamily::Portuguese => portuguese_dictionary(),
    }
}

/// Get the dictionary for a language.
pub fn dictionary(language: Language) -> Arc<FstDictionary> {
    dictionary_for_language(language.family())
}

// ========== PARSERS ==========

/// Get a parser for the given language ID and language.
///
/// Returns None if no specialized parser exists for the combination.
pub fn parser_for_prose(
    language_id: &str,
    language: Language,
    markdown_options: MarkdownOptions,
) -> Option<Box<dyn Parser>> {
    match (language_id, prose_language(&language)) {
        // Mail format
        ("mail", ProseLanguage::German) => Some(Box::new(PlainGerman)),
        ("mail", ProseLanguage::Portuguese) => Some(Box::new(PlainPortuguese)),
        ("mail", ProseLanguage::English) => Some(Box::new(PlainEnglish)),

        // Markdown/Quarto format
        ("markdown" | "quarto", ProseLanguage::German) => {
            Some(Box::new(Markdown::new_german(markdown_options)))
        }
        ("markdown" | "quarto", _) => Some(Box::new(Markdown::new(markdown_options))),

        // Org mode format
        ("org", ProseLanguage::German) => Some(Box::new(OrgMode::new_german())),
        ("org", _) => Some(Box::new(OrgMode::default())),

        // Plain text format
        ("plaintext" | "text", ProseLanguage::German) => Some(Box::new(PlainGerman)),
        ("plaintext" | "text", ProseLanguage::Portuguese) => Some(Box::new(PlainPortuguese)),
        ("plaintext" | "text", ProseLanguage::English) => Some(Box::new(PlainEnglish)),
        _ => None,
    }
}

// ========== LINTERS ==========

/// Add language-specific linters to the lint group.
///
/// Each language has its own set of rules that need to be enabled.
pub fn add_language_specific_linters(
    out: &mut LintGroup,
    language: Language,
    dictionary: Arc<impl Dictionary + 'static>,
) {
    match language {
        Language::German(_) => {
            out.add("GermanFillerWords", GermanFillerWords::default());
            out.config.set_rule_enabled("GermanFillerWords", true);
            out.add(
                "GermanSpellCheck",
                GermanSpellCheck::new(dictionary.clone()),
            );
            out.config.set_rule_enabled("GermanSpellCheck", true);
            out.add(
                "GermanNounCapitalization",
                GermanNounCapitalization::new(dictionary.clone()),
            );
            out.config
                .set_rule_enabled("GermanNounCapitalization", true);
            out.add(
                "GermanSentenceCapitalization",
                GermanSentenceCapitalization::new(dictionary.clone()),
            );
            out.config
                .set_rule_enabled("GermanSentenceCapitalization", true);
        }
        Language::Portuguese(_) => {
            out.add(
                "PortugueseSpellCheck",
                PortugueseSpellCheck::new(dictionary.clone()),
            );
            out.config.set_rule_enabled("PortugueseSpellCheck", true);
        }
        Language::English(_) => {
            // English is the default, no additional linters needed
        }
    }
}

// ========== WIR RULES ==========

/// Get the Weir rule lint group for a specific language.
///
/// This provides a centralized way to access language-specific Weir rules.
/// To add Weir rules for a new language:
/// 1. Create the Weir rule files under `language/<name>/linting/weir_rules/`
/// 2. Create a `mod.rs` under `language/<name>/linting/weir_rules/` that exposes a `lint_group()` function
/// 3. Add the language to this match statement
/// 4. Add the appropriate env vars in build.rs
pub fn weir_rules_lint_group(language: Language) -> LintGroup {
    match language {
        Language::German(_) => crate::language::german::linting::weir_rules::lint_group(),
        Language::English(_) => crate::linting::english::weir_rules::lint_group(),
        Language::Portuguese(_) => {
            // Portuguese currently has no Weir rules
            LintGroup::default()
        }
    }
}
