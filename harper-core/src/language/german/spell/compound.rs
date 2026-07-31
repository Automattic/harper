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
    // Collect words with compound flags
    let mut compound_words: Vec<&AnnotatedWord> = Vec::new();

    for word in words {
        // Check if this word has any compound flags
        let has_compound_flag = word.annotations.iter().any(|&c| is_compound_flag(c));

        if has_compound_flag {
            compound_words.push(word);
        }
    }

    // Generate compounds from all pairs
    let compound_count = compound_words.len();
    for i in 0..compound_count {
        for j in 0..compound_count {
            // Skip self-combination (word + word is rarely valid)
            if i == j {
                continue;
            }

            let first_word = &compound_words[i];
            let second_word = &compound_words[j];

            // Get compound flags for both words
            let first_flags: Vec<char> = first_word
                .annotations
                .iter()
                .filter(|&&c| is_compound_flag(c))
                .copied()
                .collect();

            let second_flags: Vec<char> = second_word
                .annotations
                .iter()
                .filter(|&&c| is_compound_flag(c))
                .copied()
                .collect();

            if first_flags.is_empty() || second_flags.is_empty() {
                continue;
            }

            // Check if the first word has adjective flag (q)
            let first_has_adj_flag = first_flags.contains(&COMPOUND_ADJ_FLAG);
            // Check if the second word has adjective flag (q)
            let second_has_adj_flag = second_flags.contains(&COMPOUND_ADJ_FLAG);

            // For adjective compounds: if either word has the q flag, create an adjective compound
            // This handles noun+adjective, adjective+noun, and adjective+adjective combinations
            if first_has_adj_flag || second_has_adj_flag {
                // Generate adjective compound word (no interfix for adjective compounds)
                let mut compound_chars: CharString = first_word.letters.clone();
                compound_chars.extend_from_slice(&second_word.letters);

                // Create metadata for the compound adjective
                // We need to add the adjective declension flags so the inflection system can generate all forms
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
            else {
                // Use the first word's first compound flag to determine interfix
                let interfix = get_interfix(first_flags[0]);

                // Generate compound word
                let mut compound_chars: CharString = first_word.letters.clone();
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
