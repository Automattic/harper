use super::{Lint, LintKind, Linter, Suggestion};
use crate::document::Document;
use crate::spell::{Dictionary, suggest_correct_spelling};
use crate::{
    CharString, CharStringExt, DialectFlags, DialectsEnum, PortugueseDialect, TokenStringExt,
};
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

impl<T: Dictionary> Linter for SpellCheck<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word in document.iter_words() {
            let word_chars = document.get_span_content(&word.span);

            if let Some(metadata) = word.kind.as_word().unwrap()
                && metadata.dialects.is_dialect_enabled(self.dialect)
                && (self.dictionary.contains_exact_word(word_chars)
                    || self.dictionary.contains_exact_word(&word_chars.to_lower()))
            {
                continue;
            };

            let mut possibilities = self.suggest_correct_spelling(word_chars);

            // If the misspelled word is capitalized, capitalize the results too.
            if let Some(mis_f) = word_chars.first()
                && mis_f.is_uppercase()
            {
                for sug_f in possibilities.iter_mut().filter_map(|w| w.first_mut()) {
                    *sug_f = sug_f.to_uppercase().next().unwrap();
                }
            }

            let suggestions: Vec<_> = possibilities
                .iter()
                .map(|sug| Suggestion::ReplaceWith(sug.to_vec()))
                .collect();

            // If there's only one suggestion, save the user a step in the GUI
            let message = if suggestions.len() == 1 {
                format!(
                    "Did you mean `{}`?",
                    possibilities.first().unwrap().iter().collect::<String>()
                )
            } else {
                format!(
                    "Did you mean to spell `{}` this way?",
                    document.get_span_content_str(&word.span)
                )
            };

            lints.push(Lint {
                span: word.span,
                lint_kind: LintKind::Spelling,
                suggestions,
                message,
                priority: 63,
            })
        }

        lints
    }

    fn description(&self) -> &'static str {
        "Looks and provides corrections for misspelled words."
    }
}

#[cfg(test)]
mod tests_portuguese {
    use super::SpellCheck;
    use crate::PortugueseDialect;
    use crate::languages::LanguageFamily;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};
    use crate::spell::FstDictionary;

    // Capitalization tests

    #[test]
    fn brasil_capitalized() {
        assert_suggestion_result(
            "A palavra brasil deveria ser capitalizada.",
            SpellCheck::new(
                FstDictionary::curated_select_language(LanguageFamily::Portuguese),
                PortugueseDialect::Brazilian,
            ),
            "A palavra Brasil deveria ser capitalizada",
            LanguageFamily::Portuguese,
        );
    }

    #[test]
    fn harper_automattic_capitalized() {
        assert_lint_count(
            "Tal qual harper e automattic.",
            SpellCheck::new(
                FstDictionary::curated_select_language(LanguageFamily::Portuguese),
                PortugueseDialect::Brazilian,
            ),
            2,
        );
    }
}
