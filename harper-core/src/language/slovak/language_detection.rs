//! Slovak language detector.
//!
//! Uses characteristic Slovak features:
//! - Special characters (á, ä, é, í, ó, ô, ú, ň, š, č, ž, ť, ď, ľ, y)
//! - Common Slovak words and articles
//! - Low English word match rate

use crate::language::LanguageDetector;
use crate::language::languages::Language;
use crate::language::slovak::dialects::SlovakDialect;
use crate::spell::{Dictionary, FstDictionary};
use crate::{Token, TokenKind};

/// Slovak language detector with high confidence due to unique characters.
#[derive(Debug)]
pub struct SlovakDetector;

impl LanguageDetector for SlovakDetector {
    fn name(&self) -> &str {
        "slovak"
    }

    fn detect(&self, toks: &[Token], source: &[char], dict: &FstDictionary) -> Option<Language> {
        let mut total_words = 0;
        let mut slovak_char_count = 0;
        let mut common_slovak_words = 0;
        let mut english_matches = 0;

        // High-confidence Slovak indicators (articles, pronouns, common verbs)
        let slovak_indicators = [
            // Personal pronouns
            "ja", "ty", "on", "ona", "ono", "my", "vy", "oni", "ony", "mňa", "teba", "ho", "ju",
            "nás", "vás", // Verb forms
            "som", "si", "je", "sme", "ste", "sú", "mám", "máš", "má", "máme", "máte", "majú",
            // Common words
            "nie", "áno", "aj", "ale", "a", "i", "alebo", "v", "z", "do", "na", "pri", "po", "pre",
            "o", "k", // Prepositions and other common words
            "od", "za", "pod", "nad", "medzi", "bez", "cez",
        ];

        for token in toks {
            match token.kind {
                TokenKind::Word(_) => {
                    total_words += 1;
                    let word_content: String = token.get_ch(source).iter().collect();

                    // Check for Slovak special characters (very high confidence)
                    // Slovak uses: á, ä, é, í, ó, ô, ú, ň, š, č, ž, ť, ď, ľ
                    if word_content.contains('á')
                        || word_content.contains('ä')
                        || word_content.contains('é')
                        || word_content.contains('í')
                        || word_content.contains('ó')
                        || word_content.contains('ô')
                        || word_content.contains('ú')
                        || word_content.contains('ň')
                        || word_content.contains('š')
                        || word_content.contains('č')
                        || word_content.contains('ž')
                        || word_content.contains('ť')
                        || word_content.contains('ď')
                        || word_content.contains('ľ')
                    {
                        slovak_char_count += 1;
                    }

                    // Check for common Slovak words
                    let lower_word = word_content.to_lowercase();
                    if slovak_indicators.contains(&lower_word.as_str()) {
                        common_slovak_words += 1;
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

        // Calculate detection scores
        let slovak_char_ratio = slovak_char_count as f64 / total_words as f64;
        let slovak_word_ratio = common_slovak_words as f64 / total_words as f64;
        let english_match_ratio = if total_words > 0 {
            english_matches as f64 / total_words as f64
        } else {
            0.0
        };

        // High confidence: Slovak special characters present
        // Characters like ň, š, č, ž, ť, ď, ľ are very specific to Slovak and related Slavic languages
        if slovak_char_ratio >= 0.01 {
            // 1%+ words have Slovak-specific characters
            return Some(Language::Slovak(SlovakDialect::Standard));
        }

        // Check if English is clearly dominant (more than 65% English words)
        if english_match_ratio >= 0.65 {
            return None; // English is clearly dominant
        }

        // Strong indicator: Many common Slovak words
        if slovak_word_ratio >= 0.20 {
            // 20%+ words are common Slovak words
            return Some(Language::Slovak(SlovakDialect::Standard));
        }

        // Medium confidence: Low English match but some Slovak words
        if english_match_ratio < 0.4 && slovak_word_ratio >= 0.08 {
            return Some(Language::Slovak(SlovakDialect::Standard));
        }

        None
    }

    fn confidence(&self) -> f64 {
        // High confidence due to unique character detection
        0.95
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::language::languages::Language;
    use crate::spell::FstDictionary;

    fn test_detection(text: &str, expected_slovak: bool) {
        let dict = FstDictionary::curated();
        let doc = Document::new_plain_english_curated(text);
        let detector = SlovakDetector;

        let result = detector.detect(doc.get_tokens(), doc.get_source(), &dict);
        assert_eq!(result.is_some(), expected_slovak, "Failed for: {}", text);
        if expected_slovak {
            assert_eq!(result.unwrap(), Language::Slovak(SlovakDialect::Standard));
        }
    }

    #[test]
    fn detects_slovak_special_chars() {
        test_detection("Ja som Šťastný. Čo robíš? Žijeme v Bratislave.", true);
    }

    #[test]
    fn detects_common_slovak_words() {
        test_detection(
            "Ja som v dome. Ty si v škole. On je v práci. My sme šťastní.",
            true,
        );
    }

    #[test]
    fn detects_mixed_slovak_english() {
        test_detection("Ja som happy v dome. Ty si tired v škole.", true);
    }

    #[test]
    fn rejects_english() {
        test_detection(
            "I am happy in the house. You are tired at school. He is at work.",
            false,
        );
    }

    #[test]
    fn rejects_short_text() {
        test_detection("Ja som", false);
    }

    #[test]
    fn detects_longer_slovak_text() {
        test_detection(
            "Ja som student na univerzite v Bratislave. \
             Ty si učiteľ a robíš v škole. \
             On je programátor a pracuje v kancelárii. \
             My sme šťastní a spokojní s našim životom.",
            true,
        );
    }
}
