//! German compound word checker using lazy decomposition.
//!
//! This module provides runtime compound word checking for German, which avoids
//! the O(n²) memory explosion of pre-generating all possible compound combinations.
//! Instead, it stores only the base words with their compound flags and checks
//! at lookup time whether a word can be decomposed into valid compound parts.
//!
//! The decomposition mirrors the productive semantics proven in
//! `GermanSpellCheck`'s fallback: any dictionary word of at least
//! [`MIN_COMPOUND_PART_LEN`] characters (or any dictionary word carrying
//! compound-formation flags, regardless of length) may act as a compound
//! element, and every standard German interfix (`""`, `s`, `n`, `en`, `er`,
//! `es`) is attempted at each boundary. Membership is resolved against the
//! base dictionary when one is injected via [`CompoundChecker::set_base_dictionary`],
//! otherwise against a casing-tolerant set built from the word list.
//!
//! Subproblems are memoized by `(segment, depth)`, turning the previously
//! exponential re-decomposition of long or misspelled words into a
//! polynomial-time scan.

use hashbrown::{HashMap, HashSet};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::CharString;
use crate::dict_word_metadata::{AdjectiveData, DictWordMetadata, NounData};
use crate::spell::rune::word_list::AnnotatedWord;
use crate::spell::{Dictionary, FstDictionary};

/// Compound word formation flags for German
const COMPOUND_FLAG_NO_INTERFIX: char = 'h';
const COMPOUND_FLAG_S_INTERFIX: char = 'i';
const COMPOUND_FLAG_N_INTERFIX: char = 'k';
const COMPOUND_FLAG_EN_INTERFIX: char = 'l';
const COMPOUND_FLAG_ER_INTERFIX: char = 'm';
const COMPOUND_FLAG_ES_INTERFIX: char = 'o';
const COMPOUND_ADJ_FLAG: char = 'q';

/// All standard German linking interfixes, tried at every compound boundary.
const STANDARD_INTERFIXES: [&str; 6] = ["", "s", "n", "en", "er", "es"];

/// The minimum length of a dictionary word that may act as a compound element
/// without carrying explicit compound-formation flags.
const MIN_COMPOUND_PART_LEN: usize = 3;

/// The maximum nesting depth of a compound decomposition (mirrors the old
/// engine's `depth > 10` cap).
const MAX_COMPOUND_DEPTH: usize = 10;

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

/// Compound checker that can determine if a word is a valid German compound
pub struct CompoundChecker {
    /// Words that participate in compounds, mapped to their compound flags.
    ///
    /// This only contains words that carry compound-formation flags; it drives
    /// the metadata distinction (adjective-vs-noun via the `q` flag) and the
    /// short-part allowance.
    compound_words: HashMap<CharString, HashSet<char>>,
    /// Every word from the word list in the casings needed for case-insensitive
    /// membership lookups when no base dictionary has been injected.
    members: HashSet<CharString>,
    /// The base dictionary to resolve element membership against, when set.
    ///
    /// When present, membership queries use this dictionary (whose lookup is
    /// case-insensitive) instead of the `members` set.
    base_dict: Option<Arc<FstDictionary>>,
    /// All compound flags for quick lookup
    compound_flags: HashSet<char>,
    /// Cache for compound check results to avoid repeated decomposition
    cache: Mutex<LruCache<CharString, bool>>,
    /// Maximum time allowed for a single compound check to prevent combinatorial explosion
    max_check_time: Duration,
}

impl std::fmt::Debug for CompoundChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompoundChecker")
            .field("flagged_words", &self.compound_words.len())
            .field("members", &self.members.len())
            .field("has_base_dict", &self.base_dict.is_some())
            .field("compound_flags", &self.compound_flags)
            .field("max_check_time", &self.max_check_time)
            .finish()
    }
}

impl Clone for CompoundChecker {
    fn clone(&self) -> Self {
        Self {
            compound_words: self.compound_words.clone(),
            members: self.members.clone(),
            base_dict: self.base_dict.clone(),
            compound_flags: self.compound_flags.clone(),
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(10000).unwrap())),
            max_check_time: self.max_check_time,
        }
    }
}

/// Insert `letters` (plus the first-letter-lowercased and first-letter-capitalized
/// variants) into `members` so case-insensitive lookups always hit.
fn insert_member_casings(members: &mut HashSet<CharString>, letters: &[char]) {
    if letters.is_empty() {
        return;
    }

    let exact: CharString = letters.iter().copied().collect();
    members.insert(exact.clone());

    let mut lowercased = exact.clone();
    if let Some(first_char) = lowercased.first_mut() {
        *first_char = first_char.to_lowercase().next().unwrap_or(*first_char);
    }
    members.insert(lowercased);

    let mut capitalized = exact.clone();
    if let Some(first_char) = capitalized.first_mut() {
        *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
    }
    members.insert(capitalized);
}

impl CompoundChecker {
    /// Create a new CompoundChecker from a list of annotated words
    pub fn new(word_list: &[AnnotatedWord]) -> Self {
        // Build compound words map from words that have compound flags
        let mut compound_words = HashMap::new();
        let mut members = HashSet::new();

        for word in word_list {
            let flags: HashSet<char> = word
                .annotations
                .iter()
                .filter(|&&c| is_compound_flag(c))
                .map(|&c| c.to_ascii_lowercase())
                .collect();

            // Every dictionary word may act as a compound element, regardless of
            // whether it carries compound-formation flags. Record the casings
            // needed for case-insensitive membership lookups.
            insert_member_casings(&mut members, &word.letters);

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
            members,
            base_dict: None,
            compound_flags: ['h', 'i', 'k', 'l', 'm', 'o', 'q']
                .iter()
                .copied()
                .collect(),
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(10000).unwrap())),
            max_check_time: Duration::from_millis(5000), // 5 second timeout
        }
    }

    /// Inject the base dictionary whose word membership drives compound
    /// decomposition. When set, element membership is resolved through
    /// `base.contains_word` (a case-insensitive lookup) instead of the word
    /// list collected by [`CompoundChecker::new`].
    pub fn set_base_dictionary(&mut self, base: Arc<FstDictionary>) {
        self.base_dict = Some(base);
        // Membership queries now go through the base dictionary, so the
        // casing-tolerant word-list set is no longer needed.
        self.members.clear();
        // Membership semantics changed, so cached decomposition results are stale.
        self.cache.lock().unwrap().clear();
    }

    /// Whether `word` is a member of the underlying dictionary.
    fn member_of(&self, word: &[char]) -> bool {
        match &self.base_dict {
            Some(base_dict) => base_dict.contains_word(word),
            None => self.members.contains(word),
        }
    }

    /// Whether `word` carries compound-formation flags in the word list.
    fn has_compound_flags(&self, word: &[char]) -> bool {
        self.get_compound_flags(word).is_some()
    }

    /// Whether a segment can participate in a compound as an element.
    ///
    /// A segment is usable iff it is a dictionary word AND it is either at
    /// least [`MIN_COMPOUND_PART_LEN`] characters long or carries compound
    /// flags. This keeps short real words such as `ei` or `öl` usable while
    /// excluding short words without compound flags (for example the
    /// preposition `zu`) and short garbage.
    fn element_usable(&self, segment: &[char]) -> bool {
        self.member_of(segment)
            && (segment.len() >= MIN_COMPOUND_PART_LEN || self.has_compound_flags(segment))
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

        // A word that is itself a plain dictionary element is not a compound.
        let result = if word.is_empty() || self.element_usable(word) {
            false
        } else {
            let start = Instant::now();
            let mut memo = HashMap::new();
            self.is_valid_segment(word, 0, &start, &mut memo)
        };

        // Cache result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(word_chars, result);
        }

        result
    }

    /// Recursively check whether `segment` decomposes into usable compound
    /// elements joined by standard German interfixes.
    ///
    /// The whole `segment` is accepted as an element only below the top level
    /// (`depth > 0`); at the top level a plain dictionary word must first be
    /// rejected by the `element_usable` guard in [`CompoundChecker::is_compound_word`].
    /// Results are memoized by `(segment, depth)` so that overlapping
    /// subproblems are evaluated at most once per top-level check.
    fn is_valid_segment(
        &self,
        segment: &[char],
        depth: usize,
        start: &Instant,
        memo: &mut HashMap<(Vec<char>, usize), bool>,
    ) -> bool {
        if start.elapsed() > self.max_check_time {
            return false; // Timeout exceeded
        }

        if depth > MAX_COMPOUND_DEPTH {
            return false;
        }

        // Below the top level, the whole sub-segment may itself be an element.
        // This precedes the minimum-length guard so that short elements carrying
        // compound flags (e.g. `ei`, `öl`) are accepted at any position.
        if depth > 0 && self.element_usable(segment) {
            return true;
        }

        if segment.len() < MIN_COMPOUND_PART_LEN {
            return false;
        }

        if let Some(&cached) = memo.get(&(segment.to_vec(), depth)) {
            return cached;
        }

        let mut valid = false;
        for split_pos in 1..segment.len() {
            let (first, rest) = segment.split_at(split_pos);

            // The left part must itself be a usable element.
            if !self.element_usable(first) {
                continue;
            }

            // Try every standard interfix at this boundary.
            for interfix in STANDARD_INTERFIXES {
                let interfix_chars: Vec<char> = interfix.chars().collect();
                let Some(after) = rest.strip_prefix(interfix_chars.as_slice()) else {
                    continue;
                };

                if self.is_valid_segment(after, depth + 1, start, memo) {
                    valid = true;
                    break;
                }
            }

            if valid {
                break;
            }
        }

        memo.insert((segment.to_vec(), depth), valid);
        valid
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

        let start = Instant::now();
        let mut memo = HashMap::new();
        self.collect_parts(word, 0, &start, &mut memo)
    }

    /// Collect the parts of the first successful decomposition of `segment`.
    ///
    /// Uses the same `element_usable` predicate and interfix probing as
    /// [`CompoundChecker::is_valid_segment`]. The interfix string is emitted as
    /// its own part (e.g. `arbeitsgeber` yields `["arbeit", "s", "geber"]`).
    /// Failed suffixes are memoized to avoid re-exploration.
    fn collect_parts(
        &self,
        segment: &[char],
        depth: usize,
        start: &Instant,
        memo: &mut HashMap<(Vec<char>, usize), Option<Vec<String>>>,
    ) -> Option<Vec<String>> {
        if start.elapsed() > self.max_check_time {
            return None;
        }

        if depth > MAX_COMPOUND_DEPTH {
            return None;
        }

        // Mirror is_valid_segment: accept short flagged elements below the top
        // level before applying the minimum-length guard.
        if depth > 0 && self.element_usable(segment) {
            return Some(vec![segment.iter().collect()]);
        }

        if segment.len() < MIN_COMPOUND_PART_LEN {
            return None;
        }

        if let Some(cached) = memo.get(&(segment.to_vec(), depth)) {
            return cached.clone();
        }

        let mut result = None;
        'search: for split_pos in 1..segment.len() {
            let (first, rest) = segment.split_at(split_pos);

            if !self.element_usable(first) {
                continue;
            }

            for interfix in STANDARD_INTERFIXES {
                let interfix_chars: Vec<char> = interfix.chars().collect();
                let Some(after) = rest.strip_prefix(interfix_chars.as_slice()) else {
                    continue;
                };

                if let Some(mut tail) = self.collect_parts(after, depth + 1, start, memo) {
                    let mut parts = vec![first.iter().collect()];
                    if !interfix.is_empty() {
                        parts.push(interfix.to_string());
                    }
                    parts.append(&mut tail);
                    result = Some(parts);
                    break 'search;
                }
            }
        }

        memo.insert((segment.to_vec(), depth), result.clone());
        result
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
        // Realistic deep-nesting fixture: every element is a full-length German-
        // looking word (no single-character elements), and a 4-5 level compound
        // must decompose recursively.
        let words = vec![
            AnnotatedWord {
                letters: "dampf".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "schiff".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "fahrt".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "kapitän".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "gesellschaft".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // 2-element compound
        assert!(checker.is_compound_word(&"dampfschiff".chars().collect::<Vec<_>>()));

        // 3-element compound
        assert!(checker.is_compound_word(&"dampfschifffahrt".chars().collect::<Vec<_>>()));

        // 4-element compound with an s-interfix
        assert!(checker.is_compound_word(&"dampfschifffahrtskapitän".chars().collect::<Vec<_>>()));

        // 5-element compound with an s-interfix
        assert!(
            checker.is_compound_word(&"dampfschifffahrtsgesellschaft".chars().collect::<Vec<_>>())
        );

        // Single fixture words are elements, not compounds.
        for word in ["dampf", "schiff", "fahrt", "kapitän", "gesellschaft"] {
            assert!(
                !checker.is_compound_word(&word.chars().collect::<Vec<_>>()),
                "{word}"
            );
        }
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
        // Fixture where every single letter is a usable (flagged) element, so a
        // word like "aaaa..." is decomposable in principle. The depth cap (and
        // the requirement that the final element be at least MIN_COMPOUND_PART_LEN
        // characters) keeps it from ever succeeding. With subproblem memoization
        // the check completes quickly instead of re-decomposing exponentially;
        // the 1ms timeout guard is never the deciding factor, but must not
        // change the outcome either.
        let words: Vec<AnnotatedWord> = (b'a'..=b'z')
            .map(|c| AnnotatedWord {
                letters: vec![c as char].into(),
                annotations: vec!['N', 'h'],
            })
            .collect();

        let mut checker = CompoundChecker::new(&words);
        checker.max_check_time = Duration::from_millis(1); // 1ms timeout

        // Very long word that would cause combinatorial explosion without memoization
        let long_word: Vec<char> = "a".repeat(100).chars().collect();

        // Should return false quickly, not hang
        let start = Instant::now();
        let result = checker.is_compound_word(&long_word);
        let elapsed = start.elapsed();

        assert!(!result);
        assert!(elapsed < Duration::from_millis(100)); // Should complete quickly
    }

    // ==================== PRODUCTIVITY TESTS ====================

    #[test]
    fn test_head_without_compound_flags() {
        // Core fix: a compound is accepted even when its head (final) element
        // carries no compound-formation flags, as long as it is a dictionary word.
        let words = vec![
            AnnotatedWord {
                letters: "haus".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "tor".chars().collect(),
                annotations: vec!['N'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // "haustor" is haus + tor, and "tor" has no compound flags at all.
        assert!(checker.is_compound_word(&"haustor".chars().collect::<Vec<_>>()));

        // Plain words are elements, not compounds.
        assert!(!checker.is_compound_word(&"haus".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"tor".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_compound_with_no_flags_anywhere() {
        // All parts are in the word list but none carries compound flags.
        let words = vec![
            AnnotatedWord {
                letters: "see".chars().collect(),
                annotations: vec!['N'],
            },
            AnnotatedWord {
                letters: "karte".chars().collect(),
                annotations: vec!['N'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        assert!(checker.is_compound_word(&"seekarte".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"see".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"karte".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_s_interfix_without_i_flag() {
        // The old engine only allowed an s-interfix when the first element
        // carried the 'i' flag. Any first element may now combine with an
        // s-interfix if the result decomposes into dictionary words.
        let words = vec![
            AnnotatedWord {
                letters: "arbeit".chars().collect(),
                annotations: vec!['N'],
            },
            AnnotatedWord {
                letters: "geber".chars().collect(),
                annotations: vec!['N'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        assert!(checker.is_compound_word(&"arbeitsgeber".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_short_flagged_element() {
        // Two-character real words (like "ei") remain usable compound elements
        // because they carry compound-formation flags.
        let words = vec![
            AnnotatedWord {
                letters: "ei".chars().collect(),
                annotations: vec!['N', 'h', 'i'],
            },
            AnnotatedWord {
                letters: "schnee".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        assert!(checker.is_compound_word(&"eischnee".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"ei".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"schnee".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_memoization_determinism_and_junk() {
        let words = vec![
            AnnotatedWord {
                letters: "see".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "karte".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // Repeated checks must return the same result (top-level cache + memo).
        let valid: Vec<char> = "seekarte".chars().collect();
        assert_eq!(
            checker.is_compound_word(&valid),
            checker.is_compound_word(&valid)
        );
        assert!(checker.is_compound_word(&valid));

        // A concatenation of two real words with a spurious inserted letter is
        // not decomposable into exactly the dictionary elements and must stay
        // rejected, deterministically.
        let junk: Vec<char> = "seeekarte".chars().collect();
        assert_eq!(
            checker.is_compound_word(&junk),
            checker.is_compound_word(&junk)
        );
        assert!(!checker.is_compound_word(&junk));
    }

    #[test]
    fn test_base_dictionary_membership() {
        use crate::spell::MutableDictionary;

        // The word list only knows "schuh" (flagged). "hersteller" is provided
        // through the injected base dictionary, as happens with the real German
        // base dictionary.
        let words = vec![AnnotatedWord {
            letters: "schuh".chars().collect(),
            annotations: vec!['N', 'h'],
        }];

        let mut checker = CompoundChecker::new(&words);

        // Without a base dictionary, "hersteller" is not a member and the
        // compound cannot be decomposed.
        assert!(!checker.is_compound_word(&"schuhhersteller".chars().collect::<Vec<_>>()));

        let mut base = MutableDictionary::new();
        base.append_word("schuh".chars().collect::<CharString>(), Default::default());
        base.append_word(
            "hersteller".chars().collect::<CharString>(),
            Default::default(),
        );
        checker.set_base_dictionary(Arc::new(base.into()));

        // With the base dictionary injected, "schuh" + "hersteller" resolves.
        assert!(checker.is_compound_word(&"schuhhersteller".chars().collect::<Vec<_>>()));
    }
}
