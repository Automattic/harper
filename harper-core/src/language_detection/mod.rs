//! Language detection system for Harper.
//!
//! This module provides the `LanguageDetector` trait and the `LanguageDetectionRegistry`
//! for detecting the language of text content. The actual detection logic is implemented
//! in the language manifest (`crate::language::manifest`).

use crate::languages::Language;
use crate::spell::FstDictionary;
use std::fmt::Debug;

/// Core trait for language detectors.
pub trait LanguageDetector: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn detect(
        &self,
        toks: &[crate::Token],
        source: &[char],
        dict: &FstDictionary,
        default_language: Language,
    ) -> Option<Language>;
    fn confidence(&self) -> f64;
}

/// Registry of all available language detectors.
///
/// This is a zero-sized type that delegates to the language manifest
/// (`crate::language::manifest::detect_language`).
pub struct LanguageDetectionRegistry;

impl LanguageDetectionRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_language(
        &self,
        source: &str,
        dict: &FstDictionary,
        default_language: Language,
    ) -> Language {
        crate::language::manifest::detect_language(source, dict, default_language)
    }
}

impl Default for LanguageDetectionRegistry {
    fn default() -> Self {
        Self
    }
}

// Public modules for each language detector
pub mod english;
pub mod german;
pub mod portuguese;

/// Check if the contents of the document are likely intended to represent English.
pub fn is_doc_likely_english(doc: &crate::Document, dict: &impl crate::spell::Dictionary) -> bool {
    is_likely_english(doc.get_tokens(), doc.get_source(), dict)
}

/// Check if given tokens are likely intended to represent English.
pub fn is_likely_english(
    toks: &[crate::Token],
    source: &[char],
    dict: &impl crate::spell::Dictionary,
) -> bool {
    use crate::TokenKind;

    let mut total_words = 0;
    let mut valid_words = 0;
    let mut punctuation = 0;
    let mut unlintable = 0;

    for token in toks {
        match token.kind {
            TokenKind::Word(_) => {
                total_words += 1;

                let word_content = token.get_ch(source);
                if dict.contains_word(word_content) {
                    valid_words += 1;
                }
            }
            TokenKind::Punctuation(_) => punctuation += 1,
            TokenKind::Unlintable => unlintable += 1,
            _ => (),
        }
    }

    if total_words <= 7 && total_words - valid_words > 0 {
        return false;
    }

    if unlintable > valid_words {
        return false;
    }

    if (punctuation as f32 * 1.25) > valid_words as f32 {
        return false;
    }

    if (valid_words as f64 / total_words as f64) < 0.7 {
        return false;
    }

    true
}
