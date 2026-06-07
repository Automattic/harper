//! Portuguese language detector.
//!
//! Uses characteristic Portuguese features:
//! - Special characters (ã, õ, á, é, í, ó, ú, â, ê, î, ô, Û, ç)
//! - Common Portuguese words and articles
//! - Low English word match rate

use crate::dialects::portuguese::PortugueseDialect;
use crate::language_detection::LanguageDetector;
use crate::languages::Language;
use crate::spell::{Dictionary, FstDictionary};
use crate::{Token, TokenKind};

/// Portuguese language detector with high confidence due to unique characters.
#[derive(Debug)]
pub struct PortugueseDetector;

impl LanguageDetector for PortugueseDetector {
    fn name(&self) -> &str {
        "portuguese"
    }

    fn detect(
        &self,
        toks: &[Token],
        source: &[char],
        dict: &FstDictionary,
        _default_language: Language,
    ) -> Option<Language> {
        let mut total_words = 0;
        let mut portuguese_char_count = 0;
        let mut common_portuguese_words = 0;
        let mut english_matches = 0;

        // High-confidence Portuguese indicators (articles, pronouns, common verbs)
        let portuguese_indicators = [
            // Definite articles
            "o",
            "a",
            "os",
            "as",
            // Indefinite articles
            "um",
            "uma",
            "uns",
            "umas",
            // Personal pronouns
            "eu",
            "tu",
            "ele",
            "ela",
            "nós",
            "vós",
            "eles",
            "elas",
            "me",
            "te",
            "se",
            "nos",
            "vos",
            // Verb forms (ser, estar, ter, ir)
            "sou",
            "és",
            "é",
            "somos",
            "sois",
            "são",
            "estou",
            "estás",
            "está",
            "estamos",
            "estais",
            "estão",
            "tenho",
            "tens",
            "tem",
            "temos",
            "tendes",
            "têm",
            "vou",
            "vais",
            "vai",
            "vamos",
            "ides",
            "vão",
            // Common words
            "que",
            "de",
            "do",
            "da",
            "no",
            "na",
            "ao",
            "aos",
            "as",
            "e",
            "ou",
            "mas",
            "por",
            "para",
            "com",
            "sem",
            "sobre",
            "não",
            "sim",
            "aqui",
            "ali",
            "agora",
            "depois",
            // Common nouns
            "pessoa",
            "coisa",
            "tempo",
            "ano",
            "dia",
            "noite",
            "mundo",
            "casa",
            "rua",
            "cidade",
            "país",
            "língua",
            "português",
        ];

        for token in toks {
            match token.kind {
                TokenKind::Word(_) => {
                    total_words += 1;
                    let word_content: String = token.get_ch(source).iter().collect();

                    // Check for Portuguese special characters (very high confidence)
                    // NOTE: ã and ç are particularly distinctive to Portuguese. õ is also used in Estonian.
                    // á, é, í, ó, ú appear in many languages but are less common in English.
                    if word_content.contains('ã')
                        || word_content.contains('õ')
                        || word_content.contains('ç')
                        || word_content.contains('á')
                        || word_content.contains('é')
                        || word_content.contains('í')
                        || word_content.contains('ó')
                        || word_content.contains('ú')
                    {
                        portuguese_char_count += 1;
                    }

                    // Check for common Portuguese words
                    let lower_word = word_content.to_lowercase();
                    if portuguese_indicators.contains(&lower_word.as_str()) {
                        common_portuguese_words += 1;
                    }

                    // Check if in English dictionary
                    if dict.contains_word(token.get_ch(source)) {
                        english_matches += 1;
                    }
                }
                TokenKind::Unlintable => {}
                _ => {}
            }
        }

        // Need minimum words for reliable detection
        if total_words < 5 {
            return None;
        }

        // High confidence if we find Portuguese special characters
        if portuguese_char_count > 0 && (portuguese_char_count as f64 / total_words as f64) > 0.3 {
            return Some(Language::Portuguese(PortugueseDialect::Brazilian));
        }

        // Medium confidence if we find multiple common Portuguese words
        // and low English match rate
        let common_word_ratio = common_portuguese_words as f64 / total_words as f64;
        let english_match_ratio = english_matches as f64 / total_words as f64;

        if common_word_ratio > 0.4 && english_match_ratio < 0.5 {
            return Some(Language::Portuguese(PortugueseDialect::Brazilian));
        }

        None
    }

    fn confidence(&self) -> f64 {
        // High confidence due to unique Portuguese characters
        0.9
    }
}

#[cfg(test)]
mod tests {
    use super::PortugueseDetector;
    use crate::Document;
    use crate::dialects::english::EnglishDialect;
    use crate::dialects::portuguese::PortugueseDialect;
    use crate::language_detection::LanguageDetector;
    use crate::languages::Language;
    use crate::parsers::PlainEnglish;
    use crate::spell::FstDictionary;

    fn test_detection(text: &str, expected_portuguese: bool) {
        let dict = FstDictionary::curated();
        let doc = Document::new(text, &PlainEnglish, &dict);
        let detector = PortugueseDetector;

        let default_lang = Language::English(EnglishDialect::American);
        let result = detector.detect(doc.get_tokens(), doc.get_source(), &dict, default_lang);
        assert_eq!(result.is_some(), expected_portuguese);
        if expected_portuguese {
            assert_eq!(
                result.unwrap(),
                Language::Portuguese(PortugueseDialect::Brazilian)
            );
        }
    }

    #[test]
    fn detects_portuguese_special_chars() {
        test_detection("O João foi à cidade de São Paulo e comprou pão.", true);
    }

    #[test]
    fn detects_common_portuguese_words() {
        test_detection(
            "João tem um cão e uma casa em São Paulo. O cão é feliz.",
            true,
        );
    }

    #[test]
    fn does_not_detect_english() {
        test_detection("The quick brown fox jumps over the lazy dog.", false);
    }

    #[test]
    fn detector_name() {
        let detector = PortugueseDetector;
        assert_eq!(detector.name(), "portuguese");
    }

    #[test]
    fn detector_confidence() {
        let detector = PortugueseDetector;
        assert!((detector.confidence() - 0.9).abs() < f64::EPSILON);
    }
}
