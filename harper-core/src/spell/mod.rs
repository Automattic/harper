//! Contains the relevant code for performing dictionary lookups and spellchecking (i.e. fuzzy
//! dictionary lookups).

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

/// Returns whether the two words are the same, expect that one is written
/// with 'ou' and the other with 'o'.
///
/// E.g. "color" and "colour"
pub(crate) fn is_ou_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len().abs_diff(b.len()) != 1 {
        return false;
    }

    let mut a_iter = a.iter();
    let mut b_iter = b.iter();

    loop {
        match (
            a_iter.next().map(char::to_ascii_lowercase),
            b_iter.next().map(char::to_ascii_lowercase),
        ) {
            (Some('o'), Some('o')) => {
                let mut a_next = a_iter.next().map(char::to_ascii_lowercase);
                let mut b_next = b_iter.next().map(char::to_ascii_lowercase);
                if a_next != b_next {
                    if a_next == Some('u') {
                        a_next = a_iter.next().map(char::to_ascii_lowercase);
                    } else if b_next == Some('u') {
                        b_next = b_iter.next().map(char::to_ascii_lowercase);
                    }

                    if a_next != b_next {
                        return false;
                    }
                }
            }
            (Some(a_char), Some(b_char)) => {
                if !a_char.eq_ignore_ascii_case(&b_char) {
                    return false;
                }
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

/// Returns whether the two words are the same, expect for a single confusion of:
///
/// - `s` and `z`. E.g."realize" and "realise"
/// - `s` and `c`. E.g. "defense" and "defence"
/// - `k` and `c`. E.g. "skepticism" and "scepticism"
pub(crate) fn is_cksz_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    if a.is_empty() {
        return true;
    }

    // the first character must be the same
    if !a[0].eq_ignore_ascii_case(&b[0]) {
        return false;
    }

    let mut found = false;
    for (a_char, b_char) in a.iter().copied().zip(b.iter().copied()) {
        let a_char = a_char.to_ascii_lowercase();
        let b_char = b_char.to_ascii_lowercase();

        if a_char != b_char {
            if (a_char == 's' && b_char == 'z')
                || (a_char == 'z' && b_char == 's')
                || (a_char == 's' && b_char == 'c')
                || (a_char == 'c' && b_char == 's')
                || (a_char == 'k' && b_char == 'c')
                || (a_char == 'c' && b_char == 'k')
            {
                if found {
                    return false;
                }
                found = true;
            } else {
                return false;
            }
        }
    }

    found
}

/// Returns whether the two words are the same, expect that one is written
/// with '-er' and the other with '-re'.
///
/// E.g. "meter" and "metre"
pub(crate) fn is_er_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len() != b.len() || a.len() <= 4 {
        return false;
    }

    let len = a.len();
    let a_suffix = [&a[len - 2], &a[len - 1]].map(char::to_ascii_lowercase);
    let b_suffix = [&b[len - 2], &b[len - 1]].map(char::to_ascii_lowercase);

    if a_suffix == ['r', 'e'] && b_suffix == ['e', 'r']
        || a_suffix == ['e', 'r'] && b_suffix == ['r', 'e']
    {
        return a[0..len - 2]
            .iter()
            .copied()
            .zip(b[0..len - 2].iter().copied())
            .all(|(a_char, b_char)| a_char.eq_ignore_ascii_case(&b_char));
    }

    false
}

/// Returns whether the two words are the same, expect that one is written
/// with 'll' and the other with 'l'.
///
/// E.g. "traveller" and "traveler"
pub(crate) fn is_ll_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len().abs_diff(b.len()) != 1 {
        return false;
    }

    let mut a_iter = a.iter();
    let mut b_iter = b.iter();

    loop {
        match (
            a_iter.next().map(char::to_ascii_lowercase),
            b_iter.next().map(char::to_ascii_lowercase),
        ) {
            (Some('l'), Some('l')) => {
                let mut a_next = a_iter.next().map(char::to_ascii_lowercase);
                let mut b_next = b_iter.next().map(char::to_ascii_lowercase);
                if a_next != b_next {
                    if a_next == Some('l') {
                        a_next = a_iter.next().map(char::to_ascii_lowercase);
                    } else if b_next == Some('l') {
                        b_next = b_iter.next().map(char::to_ascii_lowercase);
                    }

                    if a_next != b_next {
                        return false;
                    }
                }
            }
            (Some(a_char), Some(b_char)) => {
                if !a_char.eq_ignore_ascii_case(&b_char) {
                    return false;
                }
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

/// Returns whether the two words are the same, except that one is written
/// with 'ay' and the other with 'ey'.
///
/// E.g. "gray" and "grey"
pub(crate) fn is_ay_ey_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut found_ay_ey = false;
    let mut a_iter = a.iter();
    let mut b_iter = b.iter();

    while let (Some(&a_char), Some(&b_char)) = (a_iter.next(), b_iter.next()) {
        if a_char.eq_ignore_ascii_case(&b_char) {
            continue;
        }

        // Check for 'a'/'e' difference
        if (a_char.eq_ignore_ascii_case(&'a') && b_char.eq_ignore_ascii_case(&'e'))
            || (a_char.eq_ignore_ascii_case(&'e') && b_char.eq_ignore_ascii_case(&'a'))
        {
            // Check if next character is 'y' for both
            if let (Some(&a_next), Some(&b_next)) = (a_iter.next(), b_iter.next())
                && a_next.eq_ignore_ascii_case(&'y')
                && b_next.eq_ignore_ascii_case(&'y')
            {
                if found_ay_ey {
                    return false; // More than one ay/ey difference
                }
                found_ay_ey = true;
                continue;
            }
        }
        return false; // Non-ay/ey difference found
    }

    if !found_ay_ey {
        return false;
    }
    found_ay_ey
}

/// Returns whether the two words are the same, except that one is written
/// with 'ei' and the other with 'ie'.
///
/// E.g. "recieved" instead of "received", "cheif" instead of "chief"
pub(crate) fn is_ei_ie_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut found_ei_ie = false;
    let mut a_iter = a.iter();
    let mut b_iter = b.iter();

    while let (Some(&a_char), Some(&b_char)) = (a_iter.next(), b_iter.next()) {
        if a_char.eq_ignore_ascii_case(&b_char) {
            continue;
        }

        // Check for 'e' vs 'i' in first position
        if a_char.eq_ignore_ascii_case(&'e') && b_char.eq_ignore_ascii_case(&'i') {
            if let (Some(&a_next), Some(&b_next)) = (a_iter.next(), b_iter.next()) {
                // Next chars must be 'i' and 'e' respectively
                if a_next.eq_ignore_ascii_case(&'i') && b_next.eq_ignore_ascii_case(&'e') {
                    if found_ei_ie {
                        return false; // More than one ei/ie difference
                    }
                    found_ei_ie = true;
                    continue;
                }
            }
        }
        // Check for 'i' vs 'e' in first position
        else if a_char.eq_ignore_ascii_case(&'i')
            && b_char.eq_ignore_ascii_case(&'e')
            && let (Some(&a_next), Some(&b_next)) = (a_iter.next(), b_iter.next())
        {
            // Next chars must be 'e' and 'i' respectively
            if a_next.eq_ignore_ascii_case(&'e') && b_next.eq_ignore_ascii_case(&'i') {
                if found_ei_ie {
                    return false; // More than one ei/ie difference
                }
                found_ei_ie = true;
                continue;
            }
        }
        return false;
    }
    found_ei_ie
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

    // Boost common words.
    if sug.metadata.common {
        score -= 5;
    }

    // For turning words into contractions.
    if sug.word.iter().filter(|c| **c == '\'').count() == 1 {
        score -= 5;
    }

    // Detect dialect-specific variations
    if sug.edit_distance == 1
        && (is_cksz_misspelling(misspelled_word, sug.word)
            || is_ou_misspelling(misspelled_word, sug.word)
            || is_ll_misspelling(misspelled_word, sug.word)
            || is_ay_ey_misspelling(misspelled_word, sug.word))
    {
        score -= 6;
    }
    if sug.edit_distance == 2 {
        if is_ei_ie_misspelling(misspelled_word, sug.word) {
            score -= 11;
        }
        if is_er_misspelling(misspelled_word, sug.word) {
            score -= 15;
        }
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

    use crate::{
        CharStringExt, Dialect,
        linting::{
            SpellCheck,
            tests::{assert_suggestion_result, assert_top3_suggestion_result},
        },
    };

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

    // Tests for dialect-specific misspelling patterns

    // is_ou_misspelling
    #[test]
    fn suggest_color_for_colour_lowercase() {
        assert_suggestion_result(
            "colour",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "color",
        );
    }

    #[test]
    fn suggest_colour_for_color_lowercase() {
        assert_suggestion_result(
            "color",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "colour",
        );
    }

    // titlecase
    #[test]
    fn suggest_color_for_colour_titlecase() {
        assert_suggestion_result(
            "Colour",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Color",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_colour_for_color_titlecase() {
        assert_suggestion_result(
            "Color",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Colour",
        );
    }

    // all-caps
    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_color_for_colour_all_caps() {
        assert_suggestion_result(
            "COLOUR",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "COLOR",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_colour_for_color_all_caps() {
        assert_suggestion_result(
            "COLOR",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "COLOUR",
        );
    }

    // is_cksz_misspelling

    // s/z as in realise/realize
    #[test]
    fn suggest_realise_for_realize() {
        assert_suggestion_result(
            "realize",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "realise",
        );
    }

    #[test]
    fn suggest_realize_for_realise() {
        assert_suggestion_result(
            "realise",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "realize",
        );
    }

    #[test]
    fn suggest_realise_for_realize_titlecase() {
        assert_suggestion_result(
            "Realize",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Realise",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_realize_for_realise_titlecase() {
        assert_suggestion_result(
            "Realise",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Realize",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_realise_for_realize_all_caps() {
        assert_suggestion_result(
            "REALIZE",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "REALISE",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_realize_for_realise_all_caps() {
        assert_suggestion_result(
            "REALISE",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "REALIZE",
        );
    }

    // s/c as in defense/defence
    #[test]
    fn suggest_defence_for_defense() {
        assert_suggestion_result(
            "defense",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "defence",
        );
    }

    #[test]
    fn suggest_defense_for_defence() {
        assert_suggestion_result(
            "defence",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "defense",
        );
    }

    #[test]
    fn suggest_defense_for_defence_titlecase() {
        assert_suggestion_result(
            "Defense",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Defence",
        );
    }

    #[test]
    fn suggest_defence_for_defense_titlecase() {
        assert_suggestion_result(
            "Defence",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Defense",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_defense_for_defence_all_caps() {
        assert_suggestion_result(
            "DEFENSE",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "DEFENCE",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_defence_for_defense_all_caps() {
        assert_suggestion_result(
            "DEFENCE",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "DEFENSE",
        );
    }

    // k/c as in skeptic/sceptic
    #[test]
    fn suggest_sceptic_for_skeptic() {
        assert_suggestion_result(
            "skeptic",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "sceptic",
        );
    }

    #[test]
    fn suggest_skeptic_for_sceptic() {
        assert_suggestion_result(
            "sceptic",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "skeptic",
        );
    }

    #[test]
    fn suggest_sceptic_for_skeptic_titlecase() {
        assert_suggestion_result(
            "Skeptic",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Sceptic",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_skeptic_for_sceptic_titlecase() {
        assert_suggestion_result(
            "Sceptic",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Skeptic",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_skeptic_for_sceptic_all_caps() {
        assert_suggestion_result(
            "SKEPTIC",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "SCEPTIC",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_sceptic_for_skeptic_all_caps() {
        assert_suggestion_result(
            "SCEPTIC",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "SKEPTIC",
        );
    }

    // is_er_misspelling
    // as in meter/metre
    #[test]
    fn suggest_centimeter_for_centimetre() {
        assert_suggestion_result(
            "centimetre",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "centimeter",
        );
    }

    #[test]
    fn suggest_centimetre_for_centimeter() {
        assert_suggestion_result(
            "centimeter",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "centimetre",
        );
    }

    #[test]
    fn suggest_centimeter_for_centimetre_titlecase() {
        assert_suggestion_result(
            "Centimetre",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Centimeter",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_centimetre_for_centimeter_titlecase() {
        assert_suggestion_result(
            "Centimeter",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Centimetre",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_centimeter_for_centimetre_all_caps() {
        assert_suggestion_result(
            "CENTIMETRE",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "CENTIMETER",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_centimetre_for_centimeter_all_caps() {
        assert_suggestion_result(
            "CENTIMETER",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "CENTIMETRE",
        );
    }

    // is_ll_misspelling
    // as in traveller/traveler
    #[test]
    fn suggest_traveler_for_traveller() {
        assert_suggestion_result(
            "traveller",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "traveler",
        );
    }

    #[test]
    fn suggest_traveller_for_traveler() {
        assert_suggestion_result(
            "traveler",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "traveller",
        );
    }

    #[test]
    fn suggest_traveler_for_traveller_titlecase() {
        assert_suggestion_result(
            "Traveller",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Traveler",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_traveller_for_traveler_titlecase() {
        assert_suggestion_result(
            "Traveler",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Traveller",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_traveler_for_traveller_all_caps() {
        assert_suggestion_result(
            "TRAVELLER",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "TRAVELER",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_traveller_for_traveler_all_caps() {
        assert_suggestion_result(
            "TRAVELER",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "TRAVELLER",
        );
    }

    // is_ay_ey_misspelling
    // as in gray/grey

    #[test]
    fn suggest_grey_for_gray_in_non_american() {
        assert_suggestion_result(
            "I've got a gray cat.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "I've got a grey cat.",
        );
    }

    #[test]
    fn suggest_gray_for_grey_in_american() {
        assert_suggestion_result(
            "It's a greyscale photo.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "It's a grayscale photo.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_grey_for_gray_in_non_american_titlecase() {
        assert_suggestion_result(
            "I've Got a Gray Cat.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "I've Got a Grey Cat.",
        );
    }

    #[test]
    fn suggest_gray_for_grey_in_american_titlecase() {
        assert_suggestion_result(
            "It's a Greyscale Photo.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "It's a Grayscale Photo.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_grey_for_gray_in_non_american_all_caps() {
        assert_suggestion_result(
            "GRAY",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "GREY",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_gray_for_grey_in_american_all_caps() {
        assert_suggestion_result(
            "GREY",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "GRAY",
        );
    }

    // Tests for non-dialectal misspelling patterns

    // is_ei_ie_misspelling
    #[test]
    fn fix_cheif_and_recieved() {
        assert_top3_suggestion_result(
            "The cheif recieved a letter.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "The chief received a letter.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn fix_cheif_and_recieved_titlecase() {
        assert_top3_suggestion_result(
            "The Cheif Recieved a Letter.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "The Chief Received a Letter.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn fix_cheif_and_recieved_all_caps() {
        assert_top3_suggestion_result(
            "THE CHEIF RECIEVED A LETTER.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "THE CHEIF RECEIVED A LETTER.",
        );
    }
}
