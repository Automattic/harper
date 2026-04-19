//! English language detector (fallback).
//!
//! This serves as the default/fallback detector and has lower confidence
//! than language-specific detectors. It uses Harper's built-in English detection.

use harper_core::{Dialect, Token, TokenKind};
use harper_core::spell::{Dictionary, FstDictionary};
use crate::language_detection::LanguageDetector;

/// English language detector with lower confidence (fallback).
#[derive(Debug)]
pub struct EnglishDetector;

impl LanguageDetector for EnglishDetector {
    fn name(&self) -> &str {
        "english"
    }

    fn detect(&self, toks: &[Token], source: &[char], dict: &FstDictionary) -> Option<Dialect> {
        // Use Harper's built-in English detection
        let is_english = harper_core::language_detection::is_likely_english(toks, source, dict);

        if is_english {
            Some(Dialect::American)
        } else {
            None
        }
    }

    fn confidence(&self) -> f64 {
        // Lower confidence - used as fallback
        0.3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harper_core::Document;
    use harper_core::spell::FstDictionary;

    #[test]
    fn detects_english() {
        let dict = FstDictionary::curated();
        let text = "The dog plays in the garden. The cat sleeps on the sofa.";
        let doc = Document::new_plain_english_curated(text);
        let detector = EnglishDetector;

        let result = detector.detect(doc.get_tokens(), doc.get_source(), &dict);
        assert_eq!(result, Some(Dialect::American));
    }

    #[test]
    fn rejects_german() {
        let dict = FstDictionary::curated();
        let text = "Der Hund spielt im Garten. Die Katze schläft auf dem Sofa.";
        let doc = Document::new_plain_english_curated(text);
        let detector = EnglishDetector;

        let result = detector.detect(doc.get_tokens(), doc.get_source(), &dict);
        assert_eq!(result, None);
    }

    #[test]
    fn rejects_spanish() {
        let dict = FstDictionary::curated();
        let text = "Esto es español. Harper no debería marcarlo como inglés.";
        let doc = Document::new_plain_english_curated(text);
        let detector = EnglishDetector;

        let result = detector.detect(doc.get_tokens(), doc.get_source(), &dict);
        assert_eq!(result, None);
    }
}
