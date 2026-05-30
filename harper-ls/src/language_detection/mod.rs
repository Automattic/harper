//! Extensible language detection system for Harper LSP.

use harper_core::parsers::{Parser, PlainEnglish};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Token};
use std::fmt::Debug;

/// Core trait for language detectors.
pub trait LanguageDetector: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn detect(
        &self,
        toks: &[Token],
        source: &[char],
        dict: &FstDictionary,
        default_dialect: Dialect,
    ) -> Option<Dialect>;
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
        default_dialect: Dialect,
    ) -> Dialect {
        let source_chars: Vec<char> = source.chars().collect();
        // The current shared plain-text lexer is sufficient for the supported
        // Latin-script dialects. If a future language needs a different
        // tokenizer for detection, this is the seam to extend.
        let tokens = PlainEnglish.parse(&source_chars);

        if tokens.is_empty() {
            return default_dialect;
        }

        for detector in &self.detectors {
            if let Some(dialect) = detector.detect(&tokens, &source_chars, dict, default_dialect) {
                tracing::debug!(
                    "Detected language: {} using {} detector",
                    detector.name(),
                    detector.name()
                );
                return dialect;
            }
        }

        tracing::debug!("No language detected, using default dialect");
        default_dialect
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
