use crate::spell::{Dictionary, suggest_correct_spelling};
use crate::{CharString, DialectFlags, DialectsEnum, PortugueseDialect};
use lru::LruCache;
use smallvec::ToSmallVec;
use std::num::NonZero;

pub struct SpellCheck<T>
where
    T: Dictionary,
{
    dictionary: T,
    suggestion_cache: LruCache<CharString, Vec<CharString>>,
    // The language parameter might be useless because of the dictionary
    // language: Language,
    dialect: DialectsEnum,
}

impl<T: Dictionary> SpellCheck<T> {
    pub fn new(dictionary: T, dialect: PortugueseDialect) -> Self {
        Self {
            dictionary,
            suggestion_cache: LruCache::new(NonZero::new(10000).unwrap()),
            // language: Language::English(dialect),
            dialect: DialectsEnum::Portuguese(dialect),
        }
    }

    const MAX_SUGGESTIONS: usize = 3;

    fn suggest_correct_spelling(&mut self, word: &[char]) -> Vec<CharString> {
        if let Some(hit) = self.suggestion_cache.get(word) {
            hit.clone()
        } else {
            let suggestions = self.uncached_suggest_correct_spelling(word);
            self.suggestion_cache.put(word.into(), suggestions.clone());
            suggestions
        }
    }
    fn uncached_suggest_correct_spelling(&self, word: &[char]) -> Vec<CharString> {
        // Back off until we find a match.
        for dist in 2..5 {
            let suggestions: Vec<CharString> =
                suggest_correct_spelling(word, 200, dist, &self.dictionary)
                    .into_iter()
                    .filter(|v| {
                        // Ignore entries outside the configured dialect
                        self.dictionary
                            .get_word_metadata(v)
                            .unwrap()
                            .dialects
                            .is_dialect_enabled(self.dialect)
                    })
                    .map(|v| v.to_smallvec())
                    .take(Self::MAX_SUGGESTIONS)
                    .collect();

            if !suggestions.is_empty() {
                return suggestions;
            }
        }

        // no suggestions found
        Vec::new()
    }
}

#[cfg(test)]
mod tests_portuguese {
    use super::SpellCheck;
    use crate::PortugueseDialect;
    use crate::languages::{Language, LanguageFamily};
    use crate::linting::english::tests::assert_suggestion_result;
    use crate::spell::FstDictionary;

    // Capitalization tests

    #[test]
    fn brasil_capitalized() {
        let language = Language::Portuguese(PortugueseDialect::default());
        assert_suggestion_result(
            "The word brasil should be capitalized.",
            SpellCheck::new(
                FstDictionary::curated_select_language(LanguageFamily::Portuguese),
                language.into(),
            ),
            "The word Brasil should be capitalized.",
            LanguageFamily::Portuguese,
        );
    }
}
