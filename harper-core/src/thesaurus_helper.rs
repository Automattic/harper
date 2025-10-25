use crate::spell::Dictionary;
use crate::spell::FstDictionary;
use crate::{
    TokenKind,
    linting::{Suggestion, SuggestionCollectionExt},
};

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

/// Gets synonyms for a provided word, sorted by the following means:
/// - The level of difference between the provided token and that of the synonym.
/// - How often the synonym is used.
///
/// If the `thesaurus` feature is not enabled, will always return [`None`].
#[allow(unreachable_code)]
pub fn get_synonyms_sorted(word: &str, token: &TokenKind) -> Option<Vec<&'static str>> {
    #[cfg(feature = "thesaurus")]
    {
        // Sorting by frequency.
        let mut syns = harper_thesaurus::thesaurus()
            .get_synonyms_freq_sorted(word)?
            .to_owned();

        // Sorting by TokenKind difference.
        if let Some(Some(word_meta)) = token.as_word() {
            let dict = FstDictionary::curated();
            syns.sort_by_key(|syn| {
                if let Some(syn_meta) = dict.get_lexeme_metadata_str(syn) {
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
