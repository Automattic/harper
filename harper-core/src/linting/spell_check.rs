use std::num::NonZero;

use lru::LruCache;
use smallvec::ToSmallVec;

use super::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::document::Document;
use crate::spell::suggest_correct_spelling;
use crate::{CharString, CharStringExt, Dictionary, TokenStringExt};

pub struct SpellCheck<T>
where
    T: Dictionary,
{
    dictionary: T,
    word_cache: LruCache<CharString, Vec<CharString>>,
}

impl<T: Dictionary> SpellCheck<T> {
    pub fn new(dictionary: T) -> Self {
        Self {
            dictionary,
            word_cache: LruCache::new(NonZero::new(10000).unwrap()),
        }
    }
}

impl<T: Dictionary> SpellCheck<T> {
    fn cached_suggest_correct_spelling(&mut self, word: &[char]) -> Vec<CharString> {
        if let Some(hit) = self.word_cache.get(word) {
            return hit.clone();
        }

        // Back off until we find a match.
        let mut suggestions = Vec::new();
        let mut dist = 2;

        while suggestions.is_empty() && dist < 5 {
            suggestions = suggest_correct_spelling(word, 100, dist, &self.dictionary)
                .into_iter()
                .map(|v| v.to_smallvec())
                .collect();

            dist += 1;
        }

        self.word_cache.put(word.into(), suggestions.clone());

        suggestions
    }
}

impl<T: Dictionary> Linter for SpellCheck<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word in document.iter_words() {
            let word_chars = document.get_span_content(word.span);
            if self.dictionary.contains_exact_word(word_chars)
                || self.dictionary.contains_exact_word(&word_chars.to_lower())
            {
                continue;
            }

            let mut possibilities = self.cached_suggest_correct_spelling(word_chars);

            if possibilities.len() > 3 {
                possibilities.resize_with(3, || panic!());
            }

            // If the misspelled word is capitalized, capitalize the results too.
            if let Some(mis_f) = word_chars.first() {
                if mis_f.is_uppercase() {
                    for sug_f in possibilities.iter_mut().filter_map(|w| w.first_mut()) {
                        *sug_f = sug_f.to_uppercase().next().unwrap();
                    }
                }
            }

            let suggestions = possibilities
                .iter()
                .map(|word| Suggestion::ReplaceWith(word.to_vec()));

            // If there's only one suggestion, save the user a step in the GUI
            let message = if suggestions.len() == 1 {
                format!(
                    "Did you mean “{}”?",
                    possibilities.last().unwrap().iter().collect::<String>()
                )
            } else {
                format!(
                    "Did you mean to spell “{}” this way?",
                    document.get_span_content_str(word.span)
                )
            };

            lints.push(Lint {
                span: word.span,
                lint_kind: LintKind::Spelling,
                suggestions: suggestions.collect(),
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
mod tests {
    use crate::{
        FstDictionary,
        linting::tests::{assert_lint_count, assert_suggestion_result},
    };

    use super::SpellCheck;

    #[test]
    fn markdown_capitalized() {
        assert_suggestion_result(
            "The word markdown should be capitalized.",
            SpellCheck::new(FstDictionary::curated()),
            "The word Markdown should be capitalized.",
        );
    }

    #[test]
    fn harper_automattic_capitalized() {
        assert_lint_count(
            "So should harper and automattic.",
            SpellCheck::new(FstDictionary::curated()),
            2,
        );
    }
}
