//! Language registry - central integration point using LanguageModule trait.
//!
//! This module provides all orchestration functions for language support.
//! It is the only place that imports concrete language module implementations.

use std::fmt::Debug;
use std::sync::{Arc, LazyLock};

use super::languages::{Language, LanguageFamily};
use crate::spell::{Dictionary, FstDictionary};
use crate::{
    LintGroup,
    parsers::{Markdown, MarkdownOptions, OrgMode, Parser},
};

use super::english::module::EnglishModule;
use super::module::{LanguageDetector, LanguageModule};

#[cfg(feature = "de")]
use super::german::module::GermanModule;

#[cfg(feature = "pt")]
use super::portuguese::module::PortugueseModule;

#[cfg(feature = "sk")]
use super::slovak::module::SlovakModule;

/// All language detectors, sorted by confidence (highest to lowest).
#[allow(clippy::vec_init_then_push)]
static DETECTORS: LazyLock<Vec<(Box<dyn LanguageDetector>, f64)>> = LazyLock::new(|| {
    let mut detectors: Vec<(Box<dyn LanguageDetector>, f64)> = Vec::new();

    #[cfg(feature = "de")]
    detectors.push((Box::new(GermanModule::detector()), 0.95));

    #[cfg(feature = "sk")]
    detectors.push((Box::new(SlovakModule::detector()), 0.90));

    #[cfg(feature = "pt")]
    detectors.push((Box::new(PortugueseModule::detector()), 0.85));

    detectors.push((Box::new(EnglishModule::detector()), 0.30));

    detectors
});

/// Detect the language of the given source text.
pub fn detect_language(source: &str, dict: &FstDictionary, default_language: Language) -> Language {
    use crate::parsers::PlainEnglish;

    let source_chars: Vec<char> = source.chars().collect();
    let tokens = PlainEnglish.parse(&source_chars);

    if tokens.is_empty() {
        return default_language;
    }

    for (detector, _confidence) in DETECTORS.iter() {
        if let Some(language) = detector.detect(&tokens, &source_chars, dict) {
            return language;
        }
    }
    default_language
}

/// Prose languages supported by Harper for text parsing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProseLanguage {
    English,
    #[cfg(feature = "de")]
    German,
    #[cfg(feature = "pt")]
    Portuguese,
    #[cfg(feature = "sk")]
    Slovak,
}

/// Convert a Harper Language to a ProseLanguage.
pub fn prose_language(language: &Language) -> ProseLanguage {
    match language {
        Language::English(_) => ProseLanguage::English,
        #[cfg(feature = "de")]
        Language::German(_) => ProseLanguage::German,
        #[cfg(feature = "pt")]
        Language::Portuguese(_) => ProseLanguage::Portuguese,
        #[cfg(feature = "sk")]
        Language::Slovak(_) => ProseLanguage::Slovak,
    }
}

/// Get the dictionary for a language family.
pub fn dictionary_for_language(family: LanguageFamily) -> Arc<FstDictionary> {
    match family {
        LanguageFamily::English => EnglishModule::dictionary(),
        #[cfg(feature = "de")]
        LanguageFamily::German => GermanModule::dictionary(),
        #[cfg(feature = "pt")]
        LanguageFamily::Portuguese => PortugueseModule::dictionary(),
        #[cfg(feature = "sk")]
        LanguageFamily::Slovak => SlovakModule::dictionary(),
    }
}

/// Get the dictionary for a language.
pub fn dictionary(language: Language) -> Arc<FstDictionary> {
    dictionary_for_language(language.family())
}

// ========== PARSERS ==========

/// Get a parser for the given language ID and language.
pub fn parser_for_prose(
    language_id: &str,
    language: Language,
    markdown_options: MarkdownOptions,
) -> Option<Box<dyn Parser>> {
    match (language_id, prose_language(&language)) {
        ("mail", ProseLanguage::English) => Some(Box::new(EnglishModule::plain_parser())),
        #[cfg(feature = "de")]
        ("mail", ProseLanguage::German) => Some(Box::new(GermanModule::plain_parser())),
        #[cfg(feature = "pt")]
        ("mail", ProseLanguage::Portuguese) => Some(Box::new(PortugueseModule::plain_parser())),
        #[cfg(feature = "sk")]
        ("mail", ProseLanguage::Slovak) => Some(Box::new(SlovakModule::plain_parser())),

        // Markdown/Quarto format
        #[cfg(feature = "de")]
        ("markdown" | "quarto", ProseLanguage::German) => Some(Box::new(
            Markdown::with_inline_parser(markdown_options, |source| {
                GermanModule::plain_parser().parse(source)
            }),
        )),
        #[cfg(feature = "sk")]
        ("markdown" | "quarto", ProseLanguage::Slovak) => Some(Box::new(
            Markdown::with_inline_parser(markdown_options, |source| {
                SlovakModule::plain_parser().parse(source)
            }),
        )),
        #[cfg(feature = "pt")]
        ("markdown" | "quarto", ProseLanguage::Portuguese) => Some(Box::new(
            Markdown::with_inline_parser(markdown_options, |source| {
                PortugueseModule::plain_parser().parse(source)
            }),
        )),
        ("markdown" | "quarto", _) => Some(Box::new(Markdown::new(markdown_options))),

        // Org mode format
        #[cfg(feature = "de")]
        ("org", ProseLanguage::German) => Some(Box::new(OrgMode::with_inline_parser(|source| {
            GermanModule::plain_parser().parse(source)
        }))),
        #[cfg(feature = "pt")]
        ("org", ProseLanguage::Portuguese) => {
            Some(Box::new(OrgMode::with_inline_parser(|source| {
                PortugueseModule::plain_parser().parse(source)
            })))
        }
        #[cfg(feature = "sk")]
        ("org", ProseLanguage::Slovak) => Some(Box::new(OrgMode::with_inline_parser(|source| {
            SlovakModule::plain_parser().parse(source)
        }))),
        ("org", _) => Some(Box::new(OrgMode::default())),

        // Plain text format
        ("plaintext" | "text", ProseLanguage::English) => {
            Some(Box::new(EnglishModule::plain_parser()))
        }
        #[cfg(feature = "de")]
        ("plaintext" | "text", ProseLanguage::German) => {
            Some(Box::new(GermanModule::plain_parser()))
        }
        #[cfg(feature = "pt")]
        ("plaintext" | "text", ProseLanguage::Portuguese) => {
            Some(Box::new(PortugueseModule::plain_parser()))
        }
        #[cfg(feature = "sk")]
        ("plaintext" | "text", ProseLanguage::Slovak) => {
            Some(Box::new(SlovakModule::plain_parser()))
        }
        _ => None,
    }
}

// ========== LINTERS ==========

/// Add language-specific linters to the lint group.
pub fn add_language_specific_linters(
    out: &mut LintGroup,
    language: Language,
    dictionary: Arc<impl Dictionary + 'static>,
) {
    match language {
        Language::English(_dialect) => {
            let lang_group = EnglishModule::rust_lint_group(dictionary);
            out.merge_from(lang_group);
        }
        #[cfg(feature = "de")]
        Language::German(_dialect) => {
            let lang_group = GermanModule::rust_lint_group(dictionary);
            out.merge_from(lang_group);
        }
        #[cfg(feature = "pt")]
        Language::Portuguese(_dialect) => {
            let lang_group = PortugueseModule::rust_lint_group(dictionary);
            out.merge_from(lang_group);
        }
        #[cfg(feature = "sk")]
        Language::Slovak(_dialect) => {
            let lang_group = SlovakModule::rust_lint_group(dictionary);
            out.merge_from(lang_group);
        }
    }
}

// ========== WIR RULES ==========

/// Get the Weir rule lint group for a specific language.
pub fn weir_rules_lint_group(language: Language) -> LintGroup {
    match language {
        Language::English(_) => EnglishModule::weir_lint_group(),
        #[cfg(feature = "de")]
        Language::German(_) => GermanModule::weir_lint_group(),
        #[cfg(feature = "pt")]
        Language::Portuguese(_) => PortugueseModule::weir_lint_group(),
        #[cfg(feature = "sk")]
        Language::Slovak(_) => SlovakModule::weir_lint_group(),
    }
}

// ========== CURATED LINT GROUPS ==========

/// Create a new curated lint group for a specific language with a custom dictionary.
pub fn new_curated_for_language(
    _dictionary: Arc<impl Dictionary + 'static>,
    language: Language,
) -> LintGroup {
    use crate::language::module::LanguageModule;

    match language {
        Language::English(_dialect) => {
            #[allow(clippy::let_and_return)]
            let group = EnglishModule::curated_lint_group(_dialect);
            group
        }
        #[cfg(feature = "de")]
        Language::German(_dialect) => {
            use crate::language::german::linting::new_curated_german;
            new_curated_german(_dialect)
        }
        #[cfg(feature = "pt")]
        Language::Portuguese(_dialect) => {
            use crate::language::portuguese::module::PortugueseModule;

            let lang_dict = PortugueseModule::dictionary();
            let mut group = LintGroup::empty();
            group.merge_from(PortugueseModule::weir_lint_group());
            group.merge_from(PortugueseModule::rust_lint_group(lang_dict));
            group.set_all_rules_to(Some(true));
            group
        }
        #[cfg(feature = "sk")]
        Language::Slovak(_dialect) => {
            use crate::language::slovak::module::SlovakModule;

            let lang_dict = SlovakModule::dictionary();
            let mut group = LintGroup::empty();
            group.merge_from(SlovakModule::weir_lint_group());
            group.merge_from(SlovakModule::rust_lint_group(lang_dict));
            group.set_all_rules_to(Some(true));
            group
        }
    }
}
