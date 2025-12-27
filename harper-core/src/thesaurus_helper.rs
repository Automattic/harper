use crate::{
    TokenKind,
    linting::{Suggestion, SuggestionCollectionExt},
};

#[cfg(feature = "thesaurus")]
use crate::spell::{Dictionary, FstDictionary};

/// Gets synonyms for a provided word.
///
/// If the `thesaurus` feature is not enabled, will always return [`None`].
#[allow(unreachable_code)]
pub fn get_synonyms(_word: &str) -> Option<Vec<&str>> {
    #[cfg(feature = "thesaurus")]
    {
        return harper_thesaurus::thesaurus().get_synonyms(_word);
    }
    None
}

/// Gets synonyms for a provided word, sorted by the following means:
/// - The level of difference between the provided token and that of the synonym.
/// - How often the synonym is used.
///
/// If the `thesaurus` feature is not enabled, will always return [`None`].
#[allow(unreachable_code)]
pub fn get_synonyms_sorted(_word: &str, _token: &TokenKind) -> Option<Vec<&'static str>> {
    #[cfg(feature = "thesaurus")]
    {
        // Sorting by frequency.
        let mut syns = harper_thesaurus::thesaurus().get_synonyms_freq_sorted(_word)?;

        // Sorting by TokenKind difference.
        if let Some(Some(word_meta)) = _token.as_word() {
            let dict = FstDictionary::curated();
            syns.sort_by_key(|syn| {
                if let Some(syn_meta) = dict.get_word_metadata_str(syn) {
                    word_meta.difference(&syn_meta)
                } else {
                    u8::MAX
                }
            });
        }

        return Some(syns);
    }
    None
}

/// Helper method to provide synonym replacement suggestions for the provided word.
///
/// The output is sorted as in [`get_synonyms_sorted()`], which attempts to place more relevant
/// results first.
///
/// If the `thesaurus` feature isn't enabled or the word cannot be found in the thesaurus, will
/// return an empty iterator.
pub fn get_synonym_replacement_suggestions(
    word: &str,
    token: &TokenKind,
) -> impl Iterator<Item = Suggestion> {
    get_synonyms_sorted(word, token)
        .unwrap_or_default()
        .to_replace_suggestions(word.chars())
}
