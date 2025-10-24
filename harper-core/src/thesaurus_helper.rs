use crate::linting::{Suggestion, SuggestionCollectionExt};

/// Gets synonyms for a provided word.
///
/// If the `thesaurus` feature is not enabled, will always return [`None`].
#[allow(unreachable_code)]
pub fn get_synonyms(word: &str) -> Option<&'static [&'static str]> {
    #[cfg(feature = "thesaurus")]
    {
        return harper_thesaurus::thesaurus().get_synonyms(word);
    }
    None
}

/// Gets synonyms for a provided word, sorted by the frequency of their use.
///
/// If the `thesaurus` feature is not enabled, will always return [`None`].
#[allow(unreachable_code)]
pub fn get_synonyms_freq_sorted(word: &str) -> Option<Vec<&'static str>> {
    #[cfg(feature = "thesaurus")]
    {
        return harper_thesaurus::thesaurus().get_synonyms_freq_sorted(word);
    }
    None
}

/// Helper method to provide synonym replacement suggestions for the provided word.
///
/// The output is sorted based on how frequently each word/synonym is used, with words that are
/// more common appearing first.
pub fn get_synonym_replacement_suggestions(word: &str) -> impl Iterator<Item = Suggestion> {
    get_synonyms_freq_sorted(word)
        .unwrap_or_default()
        .to_replace_suggestions(word.chars())
}
