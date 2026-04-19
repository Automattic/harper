//! Extensible language detection system for Harper LSP.

use harper_core::{Dialect, Token};
use harper_core::spell::FstDictionary;
use std::fmt::Debug;

/// Core trait for language detectors.
pub trait LanguageDetector: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn detect(&self, toks: &[Token], source: &[char], dict: &FstDictionary) -> Option<Dialect>;
    fn confidence(&self) -> f64;
}

/// Registry of all available language detectors.
pub struct LanguageDetectionRegistry {
    detectors: Vec<Box<dyn LanguageDetector>>,
}

impl LanguageDetectionRegistry {
    pub fn new() -> Self {
        let mut registry = Self { detectors: Vec::new() };
        registry.register_detector(Box::new(crate::language_detection::german::GermanDetector));
        registry.register_detector(Box::new(crate::language_detection::english::EnglishDetector));
        registry
    }

    pub fn register_detector(&mut self, detector: Box<dyn LanguageDetector>) {
        self.detectors.push(detector);
        self.detectors.sort_by(|a, b| b.confidence().partial_cmp(&a.confidence()).unwrap());
    }

    pub fn detect_language(&self, source: &str, dict: &FstDictionary, default_dialect: Dialect) -> Dialect {
        use harper_core::Document;
        let doc = Document::new_plain_english_curated(source);
        let toks = doc.get_tokens();

        if toks.is_empty() {
            return default_dialect;
        }

        for detector in &self.detectors {
            if let Some(dialect) = detector.detect(toks, doc.get_source(), dict) {
                tracing::debug!("Detected language: {} using {} detector", detector.name(), detector.name());
                return dialect;
            }
        }

        tracing::debug!("No language detected, using default dialect");
        default_dialect
    }

    pub fn detector_names(&self) -> Vec<&str> {
        self.detectors.iter().map(|d| d.name()).collect()
    }
}

impl Default for LanguageDetectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Public modules for each language detector
pub mod german;
pub mod english;