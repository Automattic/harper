use std::borrow::Cow;

use itertools::{Itertools, MinMaxResult};

use crate::{CharString, CharStringExt, WordMetadata};

pub use self::dictionary::Dictionary;
pub use self::fst_dictionary::FstDictionary;
pub use self::merged_dictionary::MergedDictionary;
pub use self::mutable_dictionary::MutableDictionary;

mod dictionary;
mod fst_dictionary;
pub mod hunspell;
mod merged_dictionary;
mod mutable_dictionary;

#[derive(PartialEq, Debug, Hash, Eq)]
pub struct FuzzyMatchResult<'a> {
    pub word: &'a [char],
    pub edit_distance: u8,
    pub metadata: WordMetadata,
}

impl PartialOrd for FuzzyMatchResult<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.edit_distance.partial_cmp(&other.edit_distance)
    }
}

/// Order the suggestions to be shown to the user.
fn order_suggestions(matches: Vec<FuzzyMatchResult>) -> Vec<&[char]> {
    let mut found: Vec<&FuzzyMatchResult> = Vec::with_capacity(matches.len());
    // Often the longest and the shortest words are the most helpful, so let's push
    // them first.
    let minmax = matches.iter().position_minmax_by_key(|fmr| fmr.word.len());
    if let MinMaxResult::MinMax(a, b) = minmax {
        if a == b {
            found.push(&matches[a]);
        } else {
            found.push(&matches[a]);
            found.push(&matches[b]);
        }

        // Push the rest
        found.extend(
            matches
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != a && *i != b)
                .map(|v| v.1),
        );
    } else {
        found.extend(&matches);
    }

    // Swap the lowest edit distance word with the shortest.
    if found.len() >= 3 {
        found.swap(0, 2);
    }

    if let Some(nominal_index) = found
        .iter()
        .skip(3)
        .position(|i| i.metadata.is_proper_nominal())
    {
        found.swap(2, nominal_index + 3);
    }

    // Make commonality relevant
    found.sort_by_key(|fmr| if fmr.metadata.common { 0 } else { 1 });

    found.into_iter().map(|fmr| fmr.word).collect()
}

/// Get the closest matches in the provided [`Dictionary`] and rank them
/// Implementation is left up to the underlying dictionary.
pub fn suggest_correct_spelling<'a>(
    misspelled_word: &[char],
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &'a impl Dictionary,
) -> Vec<&'a [char]> {
    let matches: Vec<FuzzyMatchResult> = dictionary
        .fuzzy_match(misspelled_word, max_edit_dist, result_limit)
        .into_iter()
        .collect();

    order_suggestions(matches)
}

/// Convenience function over [`suggest_correct_spelling`] that does conversions
/// for you.
pub fn suggest_correct_spelling_str(
    misspelled_word: impl Into<String>,
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &impl Dictionary,
) -> Vec<String> {
    let chars: CharString = misspelled_word.into().chars().collect();
    suggest_correct_spelling(&chars, result_limit, max_edit_dist, dictionary)
        .into_iter()
        .map(|a| a.to_string())
        .collect()
}

/// Convert a given character sequence to the standard character set
/// the dictionary is in.
fn seq_to_normalized(seq: &[char]) -> Cow<'_, [char]> {
    if seq.iter().any(|c| char_to_normalized(*c) != *c) {
        Cow::Owned(seq.iter().copied().map(char_to_normalized).collect())
    } else {
        Cow::Borrowed(seq)
    }
}

fn char_to_normalized(c: char) -> char {
    match c {
        '’' => '\'',
        '‘' => '\'',
        '＇' => '\'',
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::spell::FuzzyMatchResult;

    use super::{
        Dictionary, FstDictionary, MutableDictionary, order_suggestions, seq_to_normalized,
        suggest_correct_spelling_str,
    };

    const RESULT_LIMIT: usize = 100;
    const MAX_EDIT_DIST: u8 = 2;

    #[test]
    fn normalizes_weve() {
        let word = vec!['w', 'e', '’', 'v', 'e'];
        let norm = seq_to_normalized(&word);

        assert_eq!(norm.clone(), vec!['w', 'e', '\'', 'v', 'e'])
    }

    #[test]
    fn punctation_no_duplicates() {
        let results = suggest_correct_spelling_str(
            "punctation",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        assert!(results.iter().all_unique())
    }

    /// Ensures that the suggestions are ordered taking into account commonality
    #[test]
    fn common_words_first() {
        let dict = FstDictionary::curated();
        // Select three common words
        let common_words = dict
            .words_iter()
            .filter_map(|word| {
                let metadata = dict.get_word_metadata(word).unwrap();
                if metadata.common {
                    Some(FuzzyMatchResult {
                        word,
                        edit_distance: 0,
                        metadata,
                    })
                } else {
                    None
                }
            })
            .take(3);
        // Select three uncommon words
        let uncommon_words = dict
            .words_iter()
            .filter_map(|word| {
                let metadata = dict.get_word_metadata(word).unwrap();
                if metadata.common {
                    None
                } else {
                    Some(FuzzyMatchResult {
                        word,
                        edit_distance: 0,
                        metadata,
                    })
                }
            })
            .take(3);
        // Feed the common and uncommon words into the ordering function, starting with uncommon
        // words
        let words = uncommon_words.merge(common_words).collect();
        let suggestions = order_suggestions(words);

        // Asserts that the ordering prioritizes common words
        let common_first = suggestions
            .into_iter()
            .take(3)
            .all(|word| dict.get_word_metadata(word).unwrap().common);

        assert!(common_first);
    }

    #[test]
    fn this_correction() {
        let results = suggest_correct_spelling_str(
            "Ths",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"this".to_string()));
    }

    #[test]
    fn need_correction_full() {
        let results = suggest_correct_spelling_str(
            "ned",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &MutableDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"need".to_string()));
    }

    #[test]
    fn need_correction_fst() {
        let results = suggest_correct_spelling_str(
            "ned",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"need".to_string()));
    }

    #[test]
    fn issue_624_no_duplicates() {
        let results = suggest_correct_spelling_str(
            "Semantical",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().all_unique())
    }

    #[test]
    fn issue_182_fst() {
        let results = suggest_correct_spelling_str(
            "Im",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"I'm".to_string()));
    }

    #[test]
    fn issue_182_full() {
        let results = suggest_correct_spelling_str(
            "Im",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &MutableDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"I'm".to_string()));
    }

    #[test]
    fn fst_spellcheck_hvllo() {
        let results = suggest_correct_spelling_str(
            "hvllo",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"hello".to_string()));
    }

    #[test]
    fn full_spellcheck_hvllo() {
        let results = suggest_correct_spelling_str(
            "hvllo",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &MutableDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"hello".to_string()));
    }

    #[test]
    fn fst_spellcheck_common() {
        let results = suggest_correct_spelling_str(
            "aboot",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"about".to_string()));
    }

    #[test]
    fn full_spellcheck_common() {
        let results = suggest_correct_spelling_str(
            "aboot",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &MutableDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"about".to_string()));
    }

    #[test]
    fn spellchecking_is_deterministic() {
        let results1 = suggest_correct_spelling_str(
            "hello",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );
        let results2 = suggest_correct_spelling_str(
            "hello",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );
        let results3 = suggest_correct_spelling_str(
            "hello",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        assert_eq!(results1, results2);
        assert_eq!(results1, results3);
    }
}
