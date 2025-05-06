use crate::{CharString, CharStringExt, WordMetadata};

pub use self::dictionary::Dictionary;
pub use self::fst_dictionary::FstDictionary;
pub use self::merged_dictionary::MergedDictionary;
pub use self::mutable_dictionary::MutableDictionary;
pub use self::word_id::WordId;

mod dictionary;
mod fst_dictionary;
mod merged_dictionary;
mod mutable_dictionary;
mod rune;
mod word_id;
mod word_map;

#[derive(PartialEq, Debug, Hash, Eq)]
pub struct FuzzyMatchResult<'a> {
    pub word: &'a [char],
    pub edit_distance: u8,
    pub metadata: &'a WordMetadata,
}

impl PartialOrd for FuzzyMatchResult<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.edit_distance.partial_cmp(&other.edit_distance)
    }
}

/// Scores a possible spelling suggestion based on possible relevance to the user.
///
/// Lower = better.
fn score_suggestion(misspelled_word: &[char], sug: &FuzzyMatchResult) -> i32 {
    if misspelled_word.is_empty() || sug.word.is_empty() {
        return i32::MAX;
    }

    let mut score = sug.edit_distance as i32 * 10;

    // People are much less likely to mistype the first letter.
    if misspelled_word
        .first()
        .unwrap()
        .eq_ignore_ascii_case(sug.word.first().unwrap())
    {
        score -= 10;
    }

    // If the original word is plural, the correct one probably is too.
    if *misspelled_word.last().unwrap() == 's' && *sug.word.last().unwrap() == 's' {
        score -= 5;
    }

    // For turning words into contractions.
    if sug.metadata.common {
        score -= 5;
    }

    // For turning words into contractions.
    if sug.word.iter().filter(|c| **c == '\'').count() == 1 {
        score -= 5;
    }

    score
}

/// Order the suggestions to be shown to the user.
fn order_suggestions<'b>(
    misspelled_word: &[char],
    mut matches: Vec<FuzzyMatchResult<'b>>,
) -> Vec<&'b [char]> {
    matches.sort_by_key(|v| score_suggestion(misspelled_word, v));

    matches.into_iter().map(|v| v.word).collect()
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

    order_suggestions(misspelled_word, matches)
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

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::CharStringExt;

    use super::{FstDictionary, suggest_correct_spelling_str};

    const RESULT_LIMIT: usize = 100;
    const MAX_EDIT_DIST: u8 = 2;

    #[test]
    fn normalizes_weve() {
        let word = ['w', 'e', '’', 'v', 'e'];
        let norm = word.normalized();

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

    #[test]
    fn youre_contraction() {
        assert_suggests_correction("youre", "you're");
    }

    #[test]
    fn thats_contraction() {
        assert_suggests_correction("thats", "that's");
    }

    #[test]
    fn weve_contraction() {
        assert_suggests_correction("weve", "we've");
    }

    #[test]
    fn this_correction() {
        assert_suggests_correction("ths", "this");
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
    fn issue_182() {
        assert_suggests_correction("Im", "I'm");
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

    /// Assert that the default suggestion settings result in a specific word
    /// being in the top three results for a given misspelling.
    #[track_caller]
    fn assert_suggests_correction(misspelled_word: &str, correct: &str) {
        let results = suggest_correct_spelling_str(
            misspelled_word,
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&correct.to_string()));
    }

    #[test]
    fn spellcheck_hvllo() {
        assert_suggests_correction("hvllo", "hello");
    }

    #[test]
    fn spellcheck_aout() {
        assert_suggests_correction("aout", "about");
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

    #[test]
    fn adviced_correction() {
        assert_suggests_correction("adviced", "advised");
    }

    #[test]
    fn aknowledged_correction() {
        assert_suggests_correction("aknowledged", "acknowledged");
    }

    #[test]
    fn alcaholic_correction() {
        assert_suggests_correction("alcaholic", "alcoholic");
    }

    #[test]
    fn slaves_correction() {
        assert_suggests_correction("Slaves", "Slavs");
    }

    #[test]
    fn conciousness_correction() {
        assert_suggests_correction("conciousness", "consciousness");
    }
}
