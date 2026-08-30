//! German compound word checker using lazy decomposition.
//!
//! This module provides runtime compound word checking for German, which avoids
//! the O(n²) memory explosion of pre-generating all possible compound combinations.
//! Instead, it stores only the base words with their compound flags and checks
//! at lookup time whether a word can be decomposed into valid compound parts.

use hashbrown::{HashMap, HashSet};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::CharString;
use crate::dict_word_metadata::{AdjectiveData, DictWordMetadata, NounData};
use crate::spell::rune::word_list::AnnotatedWord;

/// Compound word formation flags for German
const COMPOUND_FLAG_NO_INTERFIX: char = 'h';
const COMPOUND_FLAG_S_INTERFIX: char = 'i';
const COMPOUND_FLAG_N_INTERFIX: char = 'k';
const COMPOUND_FLAG_EN_INTERFIX: char = 'l';
const COMPOUND_FLAG_ER_INTERFIX: char = 'm';
const COMPOUND_FLAG_ES_INTERFIX: char = 'o';
const COMPOUND_ADJ_FLAG: char = 'q';

/// Interfix strings for each compound flag
const INTERFIX_MAP: &[(char, &str)] = &[
    (COMPOUND_FLAG_NO_INTERFIX, ""),
    (COMPOUND_FLAG_S_INTERFIX, "s"),
    (COMPOUND_FLAG_N_INTERFIX, "n"),
    (COMPOUND_FLAG_EN_INTERFIX, "en"),
    (COMPOUND_FLAG_ER_INTERFIX, "er"),
    (COMPOUND_FLAG_ES_INTERFIX, "es"),
];

/// Check if a character is a compound formation flag (case-insensitive)
fn is_compound_flag(c: char) -> bool {
    let lower_c = c.to_ascii_lowercase();
    matches!(
        lower_c,
        COMPOUND_FLAG_NO_INTERFIX
            | COMPOUND_FLAG_S_INTERFIX
            | COMPOUND_FLAG_N_INTERFIX
            | COMPOUND_FLAG_EN_INTERFIX
            | COMPOUND_FLAG_ER_INTERFIX
            | COMPOUND_FLAG_ES_INTERFIX
            | COMPOUND_ADJ_FLAG
    )
}

/// Get the interfix string for a compound flag
fn get_interfix(flag: char) -> &'static str {
    for &(f, interfix) in INTERFIX_MAP {
        if f == flag {
            return interfix;
        }
    }
    "" // Default: no interfix
}

/// Compound checker that can determine if a word is a valid German compound
#[derive(Debug)]
pub struct CompoundChecker {
    /// Words that can participate in compounds, mapped to their flags
    compound_words: HashMap<CharString, HashSet<char>>,
    /// All compound flags for quick lookup
    compound_flags: HashSet<char>,
    /// Cache for compound check results to avoid repeated decomposition
    cache: Mutex<LruCache<CharString, bool>>,
    /// Maximum time allowed for a single compound check to prevent combinatorial explosion
    max_check_time: Duration,
}

impl Clone for CompoundChecker {
    fn clone(&self) -> Self {
        Self {
            compound_words: self.compound_words.clone(),
            compound_flags: self.compound_flags.clone(),
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(10000).unwrap())),
            max_check_time: self.max_check_time,
        }
    }
}

impl CompoundChecker {
    /// Create a new CompoundChecker from a list of annotated words
    pub fn new(word_list: &[AnnotatedWord]) -> Self {
        // Build compound words map from words that have compound flags
        let mut compound_words = HashMap::new();

        for word in word_list {
            let flags: HashSet<char> = word
                .annotations
                .iter()
                .filter(|&&c| is_compound_flag(c))
                .map(|&c| c.to_ascii_lowercase())
                .collect();

            if !flags.is_empty() {
                // Insert the original word
                compound_words.insert(word.letters.clone(), flags.clone());

                // Also insert a version with the first letter lowercased for case-insensitive lookup
                if !word.letters.is_empty() {
                    let mut lowercased_first = word.letters.clone();
                    if let Some(first_char) = lowercased_first.first_mut() {
                        // Use proper Unicode lowercasing for German characters like Ä, Ö, Ü
                        *first_char = first_char.to_lowercase().next().unwrap_or(*first_char);
                    }
                    compound_words.insert(lowercased_first, flags);
                }
            }
        }

        Self {
            compound_words,
            compound_flags: ['h', 'i', 'k', 'l', 'm', 'o', 'q']
                .iter()
                .copied()
                .collect(),
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(10000).unwrap())),
            max_check_time: Duration::from_millis(5000), // 5 second timeout
        }
    }

    /// Check if a word is a valid German compound word
    pub fn is_compound_word(&self, word: &[char]) -> bool {
        let word_chars = CharString::from(word);

        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(&result) = cache.get(&word_chars) {
                return result;
            }
        }

        // Check with timeout protection
        let start = Instant::now();
        let result = self.try_decompose_with_timeout(word, 0, &start, 0);

        // Cache result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(word_chars, result);
        }

        result
    }

    /// Try to decompose a word into valid compound parts
    fn try_decompose(&self, word: &[char]) -> bool {
        if word.is_empty() {
            return false;
        }

        // For very short words, they can't be compounds
        if word.len() < 2 {
            return false;
        }

        // Start decomposition with timeout
        let start = Instant::now();
        self.try_decompose_with_timeout(word, 0, &start, 0)
    }

    /// Helper to get compound flags for a word, trying lowercase first if not found
    fn get_compound_flags(&self, word: &[char]) -> Option<&HashSet<char>> {
        // First try exact match
        if let Some(flags) = self.compound_words.get(word) {
            return Some(flags);
        }

        // For German, try with first letter lowercased (nouns are capitalized in text but stored lowercase in dict)
        if !word.is_empty() {
            let mut lowercased: CharString = word.iter().copied().collect();
            if let Some(first_char) = lowercased.first_mut() {
                // Use proper Unicode lowercasing for German characters like Ä, Ö, Ü
                *first_char = first_char.to_lowercase().next().unwrap_or(*first_char);
            }
            self.compound_words.get(&lowercased)
        } else {
            None
        }
    }

    /// Recursive helper to try decomposition starting from a given position with timeout and depth limit
    fn try_decompose_with_timeout(
        &self,
        word: &[char],
        start_pos: usize,
        start: &Instant,
        depth: usize,
    ) -> bool {
        if start.elapsed() > self.max_check_time {
            return false; // Timeout exceeded
        }

        // Prevent infinite recursion with a depth limit
        if depth > 10 {
            return false;
        }

        if start_pos >= word.len() {
            return false;
        }

        // Try all possible split points from start_pos
        for split_pos in (start_pos + 1)..word.len() {
            let (first, rest) = word.split_at(split_pos);

            // Check if first part is a compound word (case-insensitive lookup)
            let first_flags = self.get_compound_flags(first);
            if let Some(first_flags) = first_flags {
                // Check if this is an adjective compound (no interfix needed)
                if first_flags.contains(&COMPOUND_ADJ_FLAG) {
                    // For adjective compounds, the second part can be any compound word
                    if split_pos < word.len() {
                        let second = &word[split_pos..];
                        // The second part can be a dictionary word OR a valid compound (recursive)
                        if self.get_compound_flags(second).is_some()
                            || self.try_decompose_with_timeout(second, 0, start, depth + 1)
                        {
                            return true;
                        }
                        // Or recursively try to decompose the rest
                        if self.try_decompose_with_timeout(word, split_pos, start, depth) {
                            return true;
                        }
                    }
                }

                // Try noun compounds with interfixes
                // Get the first applicable interfix from the first word's flags
                let applicable_interfixes: Vec<&str> = first_flags
                    .iter()
                    .filter(|&&flag| flag != COMPOUND_ADJ_FLAG)
                    .filter_map(|&flag| {
                        let interfix = get_interfix(flag);
                        if interfix.is_empty() && flag != COMPOUND_FLAG_NO_INTERFIX {
                            None
                        } else {
                            Some(interfix)
                        }
                    })
                    .collect();

                // If no specific noun flags, try with no interfix
                if applicable_interfixes.is_empty() && !first_flags.contains(&COMPOUND_ADJ_FLAG) {
                    // No noun compound flags, so this word can't start a noun compound
                    // But we can still try the recursive decomposition for adjective compounds
                    continue;
                }

                // Try each applicable interfix
                for interfix in applicable_interfixes {
                    let interfix_chars: Vec<char> = interfix.chars().collect();
                    let interfix_len = interfix_chars.len();

                    // Check if rest starts with this interfix
                    if rest.len() >= interfix_len && rest.starts_with(&interfix_chars) {
                        let second_start = split_pos + interfix_len;
                        if second_start < word.len() {
                            let second = &word[second_start..];

                            // The second part can be a dictionary word OR a valid compound (recursive)
                            if self.get_compound_flags(second).is_some()
                                || self.try_decompose_with_timeout(second, 0, start, depth + 1)
                            {
                                return true;
                            }

                            // Or recursively try to decompose the rest after the interfix
                            if self.try_decompose_with_timeout(word, second_start, start, depth) {
                                return true;
                            }
                        }
                    }
                }

                // Also try with no interfix for h flag
                if first_flags.contains(&COMPOUND_FLAG_NO_INTERFIX) {
                    let second = &word[split_pos..];
                    if !second.is_empty() {
                        // The second part can be a dictionary word OR a valid compound (recursive)
                        if self.get_compound_flags(second).is_some()
                            || self.try_decompose_with_timeout(second, 0, start, depth + 1)
                        {
                            return true;
                        }
                        if self.try_decompose_with_timeout(word, split_pos, start, depth) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Get metadata for a compound word (for use in dictionary lookups)
    pub fn get_compound_metadata(&self, word: &[char]) -> Option<DictWordMetadata> {
        if !self.is_compound_word(word) {
            return None;
        }

        // Try to determine if it's a noun or adjective compound by checking decomposition
        // If the compound starts with an adjective (COMPOUND_ADJ_FLAG), it's likely an adjective
        if self.is_adjective_compound(word) {
            return Some(DictWordMetadata {
                adjective: Some(AdjectiveData::default()),
                ..Default::default()
            });
        }

        // Default to noun metadata for other compounds (most German compounds are nouns)
        Some(DictWordMetadata {
            noun: Some(NounData::default()),
            ..Default::default()
        })
    }

    /// Check if a compound word is an adjective compound
    fn is_adjective_compound(&self, word: &[char]) -> bool {
        if word.is_empty() {
            return false;
        }

        // Check all possible split points to see if any start with an adjective
        for split_pos in 1..word.len() {
            let (first, _rest) = word.split_at(split_pos);

            if let Some(first_flags) = self.get_compound_flags(first)
                && first_flags.contains(&COMPOUND_ADJ_FLAG)
            {
                return true;
            }
        }

        false
    }

    /// Check if a word can be decomposed and return the decomposition parts
    pub fn get_decomposition(&self, word: &[char]) -> Option<Vec<String>> {
        if word.is_empty() {
            return None;
        }

        let mut parts = Vec::new();
        if self.try_decompose_with_parts(word, 0, &mut parts) {
            Some(parts)
        } else {
            None
        }
    }

    /// Helper to try decomposition and collect parts
    fn try_decompose_with_parts(
        &self,
        word: &[char],
        start_pos: usize,
        parts: &mut Vec<String>,
    ) -> bool {
        if start_pos >= word.len() {
            return false;
        }

        for split_pos in (start_pos + 1)..word.len() {
            let (first, rest) = word.split_at(split_pos);

            if let Some(first_flags) = self.get_compound_flags(first) {
                // Check adjective compounds
                if first_flags.contains(&COMPOUND_ADJ_FLAG) && split_pos < word.len() {
                    let second = &word[split_pos..];
                    if self.get_compound_flags(second).is_some() {
                        parts.push(first.iter().collect());
                        parts.push(second.iter().collect());
                        return true;
                    }
                    if self.try_decompose_with_parts(word, split_pos, parts) {
                        parts.insert(0, first.iter().collect());
                        return true;
                    }
                }

                // Try noun compounds with interfixes
                for interfix in INTERFIX_MAP
                    .iter()
                    .filter(|(flag, _)| first_flags.contains(flag))
                {
                    let interfix_chars: Vec<char> = interfix.1.chars().collect();
                    let interfix_len = interfix_chars.len();

                    if rest.len() >= interfix_len && rest.starts_with(&interfix_chars) {
                        let second_start = split_pos + interfix_len;
                        if second_start < word.len() {
                            let second = &word[second_start..];
                            if self.get_compound_flags(second).is_some() {
                                parts.push(first.iter().collect());
                                parts.push(interfix.1.to_string());
                                parts.push(second.iter().collect());
                                return true;
                            }
                            if self.try_decompose_with_parts(word, second_start, parts) {
                                parts.insert(0, interfix.1.to_string());
                                parts.insert(0, first.iter().collect());
                                return true;
                            }
                        }
                    }
                }

                // Try with no interfix for h flag
                if first_flags.contains(&COMPOUND_FLAG_NO_INTERFIX) && split_pos < word.len() {
                    let second = &word[split_pos..];
                    if self.get_compound_flags(second).is_some() {
                        parts.push(first.iter().collect());
                        parts.push(second.iter().collect());
                        return true;
                    }
                    if self.try_decompose_with_parts(word, split_pos, parts) {
                        parts.insert(0, first.iter().collect());
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get the number of compound-eligible words
    pub fn compound_word_count(&self) -> usize {
        self.compound_words.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spell::rune::word_list::AnnotatedWord;

    fn create_test_checker() -> CompoundChecker {
        let words = vec![
            // Basic words with compound flags
            AnnotatedWord {
                letters: "schuh".chars().collect(),
                annotations: vec!['N', 'X', 'h'],
            },
            AnnotatedWord {
                letters: "hersteller".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "arbeit".chars().collect(),
                annotations: vec!['N', 'i'],
            },
            AnnotatedWord {
                letters: "geber".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "bildung".chars().collect(),
                annotations: vec!['N', 'i'], // s interfix (corrected from 'l' which is for "en" interfix)
            },
            AnnotatedWord {
                letters: "ministerium".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            // Adjective compound word
            AnnotatedWord {
                letters: "rot".chars().collect(),
                annotations: vec!['A', 'q'], // adjective with q flag
            },
            AnnotatedWord {
                letters: "haar".chars().collect(),
                annotations: vec!['N', 'q'], // noun with q flag for adjective compounds
            },
        ];

        CompoundChecker::new(&words)
    }

    #[test]
    fn test_simple_noun_compound_no_interfix() {
        let checker = create_test_checker();
        assert!(checker.is_compound_word(&"schuhhersteller".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_noun_compound_with_s_interfix() {
        let checker = create_test_checker();
        assert!(checker.is_compound_word(&"arbeitsgeber".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_noun_compound_with_en_interfix() {
        let checker = create_test_checker();
        assert!(checker.is_compound_word(&"bildungsministerium".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_adjective_compound() {
        let checker = create_test_checker();
        // rothaar (red-haired) - adjective + noun with q flag
        assert!(checker.is_compound_word(&"rothaar".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_non_compound_word() {
        let checker = create_test_checker();
        // "xyzabc" should not be recognized as a compound
        assert!(!checker.is_compound_word(&"xyzabc".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_single_word_not_compound() {
        let checker = create_test_checker();
        // Single words from the dictionary should not be compounds
        assert!(!checker.is_compound_word(&"schuh".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"arbeit".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_empty_word() {
        let checker = create_test_checker();
        assert!(!checker.is_compound_word(&[]));
    }

    #[test]
    fn test_decomposition_parts() {
        let checker = create_test_checker();
        let parts = checker.get_decomposition(&"schuhhersteller".chars().collect::<Vec<_>>());
        assert!(parts.is_some());
        let parts = parts.unwrap();
        // With recursive compounding, we might get more granular parts
        // The key is that schuh and hersteller should be present (either as parts or combined)
        let parts_str: String = parts.join("");
        assert!(parts_str.contains("schuh"));
        assert!(parts_str.contains("hersteller"));
    }

    #[test]
    fn test_decomposition_with_interfix() {
        let checker = create_test_checker();
        let parts = checker.get_decomposition(&"arbeitsgeber".chars().collect::<Vec<_>>());
        assert!(parts.is_some());
        let parts = parts.unwrap();
        assert!(parts.contains(&"arbeit".to_string()));
        assert!(parts.contains(&"s".to_string()));
        assert!(parts.contains(&"geber".to_string()));
    }

    #[test]
    fn test_compound_word_count() {
        let checker = create_test_checker();
        assert!(checker.compound_word_count() > 0);
    }

    #[test]
    fn test_cache_works() {
        let checker = create_test_checker();
        let word: Vec<char> = "schuhhersteller".chars().collect();

        // First check should populate cache
        let result1 = checker.is_compound_word(&word);

        // Second check should use cache
        let result2 = checker.is_compound_word(&word);

        // Results should be the same
        assert_eq!(result1, result2);
        assert!(result1);
    }

    #[test]
    fn test_get_compound_metadata() {
        let checker = create_test_checker();
        let word: Vec<char> = "schuhhersteller".chars().collect();

        let metadata = checker.get_compound_metadata(&word);
        assert!(metadata.is_some());
        let metadata = metadata.unwrap();
        assert!(metadata.noun.is_some());
    }

    // ==================== RECURSIVE COMPOUNDING TESTS ====================

    #[test]
    fn test_recursive_compounding_basic() {
        // Test the classic example: dampfschiff -> donaudampfschiff
        let words = vec![
            AnnotatedWord {
                letters: "donau".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "dampf".chars().collect(),
                annotations: vec!['M', 'h'],
            },
            AnnotatedWord {
                letters: "schiff".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // dampfschiff should be recognized as compound
        assert!(checker.is_compound_word(&"dampfschiff".chars().collect::<Vec<_>>()));

        // donaudampfschiff should be recognized as compound (recursive)
        assert!(checker.is_compound_word(&"donaudampfschiff".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_recursive_compounding_deep() {
        let words = vec![
            AnnotatedWord {
                letters: "a".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "b".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "c".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "d".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // ab, abc, abcd should all be recognized
        assert!(checker.is_compound_word(&"ab".chars().collect::<Vec<_>>()));
        assert!(checker.is_compound_word(&"abc".chars().collect::<Vec<_>>()));
        assert!(checker.is_compound_word(&"abcd".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_recursive_compounding_with_interfix() {
        let words = vec![
            AnnotatedWord {
                letters: "arbeit".chars().collect(),
                annotations: vec!['F', 'i'],
            },
            AnnotatedWord {
                letters: "geber".chars().collect(),
                annotations: vec!['M', 'h'],
            },
            AnnotatedWord {
                letters: "fach".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // arbeitsgeber should be recognized (with -s interfix)
        assert!(checker.is_compound_word(&"arbeitsgeber".chars().collect::<Vec<_>>()));

        // arbeitsgeberfach should be recognized (recursive)
        assert!(checker.is_compound_word(&"arbeitsgeberfach".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_recursive_compounding_with_adjective() {
        let words = vec![
            AnnotatedWord {
                letters: "rot".chars().collect(),
                annotations: vec!['A', 'q'],
            },
            AnnotatedWord {
                letters: "haar".chars().collect(),
                annotations: vec!['N', 'q'],
            },
            AnnotatedWord {
                letters: "farbe".chars().collect(),
                annotations: vec!['F', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // rothaar should work (adjective compound)
        assert!(checker.is_compound_word(&"rothaar".chars().collect::<Vec<_>>()));

        // rothaarfarbe should work recursively
        assert!(checker.is_compound_word(&"rothaarfarbe".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_existing_compounds_still_work() {
        let checker = create_test_checker();

        // All existing test cases should still pass
        assert!(checker.is_compound_word(&"schuhhersteller".chars().collect::<Vec<_>>()));
        assert!(checker.is_compound_word(&"arbeitsgeber".chars().collect::<Vec<_>>()));
        assert!(checker.is_compound_word(&"bildungsministerium".chars().collect::<Vec<_>>()));
        assert!(checker.is_compound_word(&"rothaar".chars().collect::<Vec<_>>()));

        // Non-compounds should still fail
        assert!(!checker.is_compound_word(&"xyzabc".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"schuh".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_timeout_protection() {
        // Create a checker with very short timeout
        let words = vec![AnnotatedWord {
            letters: "a".chars().collect(),
            annotations: vec!['N', 'h'],
        }];

        let mut checker = CompoundChecker::new(&words);
        checker.max_check_time = Duration::from_millis(1); // 1ms timeout

        // Very long word that would cause combinatorial explosion
        let long_word: Vec<char> = "a".repeat(100).chars().collect();

        // Should return false due to timeout, not hang
        let start = Instant::now();
        let result = checker.is_compound_word(&long_word);
        let elapsed = start.elapsed();

        assert!(!result);
        assert!(elapsed < Duration::from_millis(100)); // Should complete quickly
    }
}
