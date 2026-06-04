//! Extensible language detection system for Harper LSP.

use crate::Token;
use crate::languages::Language;
use crate::parsers::{Parser, PlainEnglish};
use crate::spell::FstDictionary;
use std::fmt::Debug;

/// Core trait for language detectors.
pub trait LanguageDetector: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn detect(
        &self,
        toks: &[Token],
        source: &[char],
        dict: &FstDictionary,
        default_language: Language,
    ) -> Option<Language>;
    fn confidence(&self) -> f64;
}

/// Registry of all available language detectors.
pub struct LanguageDetectionRegistry {
    detectors: Vec<Box<dyn LanguageDetector>>,
}

impl LanguageDetectionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            detectors: Vec::new(),
        };
        registry.register_detector(Box::new(crate::language_detection::german::GermanDetector));
        registry.register_detector(Box::new(
            crate::language_detection::portuguese::PortugueseDetector,
        ));
        registry.register_detector(Box::new(
            crate::language_detection::english::EnglishDetector,
        ));
        registry
    }

    pub fn register_detector(&mut self, detector: Box<dyn LanguageDetector>) {
        self.detectors.push(detector);
        self.detectors
            .sort_by(|a, b| b.confidence().partial_cmp(&a.confidence()).unwrap());
    }

    pub fn detect_language(
        &self,
        source: &str,
        dict: &FstDictionary,
        default_language: Language,
    ) -> Language {
        let source_chars: Vec<char> = source.chars().collect();
        // The current shared plain-text lexer is sufficient for the supported
        // Latin-script languages. If a future language needs a different
        // tokenizer for detection, this is the seam to extend.
        let tokens = PlainEnglish.parse(&source_chars);

        if tokens.is_empty() {
            return default_language;
        }

        for detector in &self.detectors {
            if let Some(language) = detector.detect(&tokens, &source_chars, dict, default_language)
            {
                return language;
            }
        }
        default_language
    }
}

impl Default for LanguageDetectionRegistry {
    fn default() -> Self {
        Self::new()
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
