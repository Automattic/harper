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

pub fn get_synonym_replacement_suggestions(word: &str) -> impl Iterator<Item = Suggestion> {
    get_synonyms(word)
        .unwrap_or_default()
        .to_replace_suggestions(word.chars())
}
