//! German-specific compound word generation.
//!
//! This module provides compound word formation for German, which uses
//! specific interfix flags (h, i, k, l, m, o) to generate valid compound nouns,
//! and the q flag for compound adjective participation.

use hashbrown::HashSet;

use crate::dict_word_metadata::AdjectiveData;
use crate::spell::rune::word_list::AnnotatedWord;
use crate::spell::word_map::{WordMap, WordMapEntry};
use crate::{CharString, DictWordMetadata, NounData};

/// Compound word formation flags (using lowercase to avoid conflicts with properties)
/// These are used to identify words that can participate in compound formation in German
pub const COMPOUND_FLAG_NO_INTERFIX: char = 'h';
pub const COMPOUND_FLAG_S_INTERFIX: char = 'i';
pub const COMPOUND_FLAG_N_INTERFIX: char = 'k';
pub const COMPOUND_FLAG_EN_INTERFIX: char = 'l';
pub const COMPOUND_FLAG_ER_INTERFIX: char = 'm';
pub const COMPOUND_FLAG_ES_INTERFIX: char = 'o';
pub const COMPOUND_ADJ_FLAG: char = 'q';

/// Interfix strings for each compound flag
const INTERFIX_MAP: &[(char, &str)] = &[
    (COMPOUND_FLAG_NO_INTERFIX, ""),
    (COMPOUND_FLAG_S_INTERFIX, "s"),
    (COMPOUND_FLAG_N_INTERFIX, "n"),
    (COMPOUND_FLAG_EN_INTERFIX, "en"),
    (COMPOUND_FLAG_ER_INTERFIX, "er"),
    (COMPOUND_FLAG_ES_INTERFIX, "es"),
];

/// Generate compound words from a list of annotated words for German.
///
/// This function processes words that have compound formation flags (h, i, k, l, m, o, q)
/// and generates compound words by combining them with appropriate interfixes.
///
/// # Arguments
/// * `words` - List of annotated words from the German dictionary
/// * `word_map` - WordMap to which generated compounds will be added
pub fn generate_compound_words(words: &[AnnotatedWord], word_map: &mut WordMap) {
    // Collect words with compound flags and pre-compute their flags
    let mut compound_words: Vec<CompoundWordInfo> = Vec::new();

    for word in words {
        // Check if this word has any compound flags
        let flags: Vec<char> = word
            .annotations
            .iter()
            .filter(|&&c| is_compound_flag(c))
            .copied()
            .collect();

        if !flags.is_empty() {
            compound_words.push(CompoundWordInfo { word, flags });
        }
    }

    let compound_count = compound_words.len();

    // Sort by word length descending so we can skip long compounds early
    // Actually, for O(n^2) we want to process shorter words first to avoid generating
    // very long compounds. But the order doesn't matter for correctness.

    // For performance, we'll use a more efficient approach:
    // 1. Pre-compute all compound flags
    // 2. Skip pairs where combined length exceeds a reasonable limit
    // 3. Use early filtering to avoid unnecessary string operations
    //
    // Note: German can have very long compounds, but most common compounds are < 40 chars.
    // We use a conservative limit to avoid generating millions of unlikely compounds.
    const MAX_COMPOUND_LENGTH: usize = 35; // Most common German compounds are under 35 chars

    for i in 0..compound_count {
        let first = &compound_words[i];
        let first_word = first.word;
        let first_flags = &first.flags;
        let first_len = first_word.letters.len();

        // Skip if first word is too long to form reasonable compounds
        if first_len >= MAX_COMPOUND_LENGTH {
            continue;
        }

        // Check if the first word has adjective flag (q)
        let first_has_adj_flag = first_flags.contains(&COMPOUND_ADJ_FLAG);

        // Pre-compute the interfix for noun compounds
        let interfix = if first_has_adj_flag {
            None
        } else {
            Some(get_interfix(first_flags[0]))
        };

        let interfix_len = interfix.map_or(0, |s| s.chars().count());
        let min_second_len = MAX_COMPOUND_LENGTH.saturating_sub(first_len + interfix_len);

        for j in 0..compound_count {
            // Skip self-combination (word + word is rarely valid)
            if i == j {
                continue;
            }

            let second = &compound_words[j];
            let second_word = second.word;
            let second_flags = &second.flags;
            let second_len = second_word.letters.len();

            if second_flags.is_empty() {
                continue;
            }

            // Check if the second word has adjective flag (q)
            let second_has_adj_flag = second_flags.contains(&COMPOUND_ADJ_FLAG);

            // Fast path: check combined length before doing expensive operations
            if first_len + interfix_len + second_len > MAX_COMPOUND_LENGTH {
                continue;
            }

            // For adjective compounds: if either word has the q flag, create an adjective compound
            // This handles noun+adjective, adjective+noun, and adjective+adjective combinations
            if first_has_adj_flag || second_has_adj_flag {
                // Generate adjective compound word (no interfix for adjective compounds)
                let mut compound_chars = CharString::with_capacity(first_len + second_len);
                compound_chars.extend_from_slice(&first_word.letters);
                compound_chars.extend_from_slice(&second_word.letters);

                // Create metadata for the compound adjective
                let compound_meta = DictWordMetadata {
                    adjective: Some(AdjectiveData::default()),
                    ..Default::default()
                };

                // Add to word map if not already present
                let compound_str: String = compound_chars.iter().collect();
                if !word_map.contains_str(&compound_str) {
                    word_map.insert(WordMapEntry {
                        canonical_spelling: compound_chars,
                        metadata: compound_meta,
                    });
                }
            }
            // For noun compounds: only when neither word has adjective flag
            else if let Some(interfix) = interfix {
                // Generate compound word
                let mut compound_chars =
                    CharString::with_capacity(first_len + interfix_len + second_len);
                compound_chars.extend_from_slice(&first_word.letters);
                compound_chars.extend(interfix.chars());
                compound_chars.extend_from_slice(&second_word.letters);

                // Create metadata for the compound
                let compound_meta = DictWordMetadata {
                    noun: Some(NounData {
                        is_proper: None,
                        is_singular: None,
                        is_plural: None,
                        is_countable: None,
                        is_mass: None,
                        is_possessive: None,
                    }),
                    ..Default::default()
                };

                // Add to word map if not already present
                let compound_str: String = compound_chars.iter().collect();
                if !word_map.contains_str(&compound_str) {
                    word_map.insert(WordMapEntry {
                        canonical_spelling: compound_chars,
                        metadata: compound_meta,
                    });
                }
            }
        }
    }
}

/// Helper struct to store pre-computed compound word information
struct CompoundWordInfo<'a> {
    word: &'a AnnotatedWord,
    flags: Vec<char>,
}

/// Check if a character is a compound formation flag
fn is_compound_flag(c: char) -> bool {
    matches!(
        c,
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

/// Parse compound flags from a word's annotations
pub fn get_compound_flags(annotations: &[char]) -> HashSet<char> {
    annotations
        .iter()
        .filter(|&&c| is_compound_flag(c))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spell::word_map::WordMap;

    #[test]
    fn test_is_compound_flag() {
        assert!(is_compound_flag('h'));
        assert!(is_compound_flag('i'));
        assert!(is_compound_flag('k'));
        assert!(is_compound_flag('l'));
        assert!(is_compound_flag('m'));
        assert!(is_compound_flag('o'));
        assert!(!is_compound_flag('n'));
        assert!(!is_compound_flag('x'));
    }

    #[test]
    fn test_get_interfix() {
        assert_eq!(get_interfix('h'), "");
        assert_eq!(get_interfix('i'), "s");
        assert_eq!(get_interfix('k'), "n");
        assert_eq!(get_interfix('l'), "en");
        assert_eq!(get_interfix('m'), "er");
        assert_eq!(get_interfix('o'), "es");
        assert_eq!(get_interfix('x'), "");
    }

    #[test]
    fn test_generate_simple_compound() {
        let words = vec![
            AnnotatedWord {
                letters: "schuh".chars().collect(),
                annotations: vec!['N', 'X', 'h'],
            },
            AnnotatedWord {
                letters: "hersteller".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let mut word_map = WordMap::default();
        generate_compound_words(&words, &mut word_map);

        assert!(word_map.contains_str("schuhhersteller"));
    }

    #[test]
    fn test_generate_compound_with_s_interfix() {
        let words = vec![
            AnnotatedWord {
                letters: "arbeit".chars().collect(),
                annotations: vec!['N', 'i'],
            },
            AnnotatedWord {
                letters: "geber".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let mut word_map = WordMap::default();
        generate_compound_words(&words, &mut word_map);

        // arbeit + s + geber = arbeitengeber (using s interfix)
        assert!(word_map.contains_str("arbeitsgeber"));
    }
}
