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

impl<T: Dictionary> Linter for SpellCheck<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word in document.iter_words() {
            let word_chars = document.get_span_content(&word.span);

            // Look the word up in our own dictionary rather than trusting the token's metadata,
            // which may have been produced by a dictionary of a different language.
            let lower = word_chars.to_lower();
            let known_form = if self.dictionary.contains_exact_word(word_chars) {
                Some(word_chars)
            } else if self.dictionary.contains_exact_word(&lower) {
                Some(lower.as_ref())
            } else {
                None
            };

            if let Some(form) = known_form
                && let Some(metadata) = self.dictionary.get_word_metadata(form)
                && metadata.dialects.is_dialect_enabled(self.dialect)
            {
                continue;
            };

            let mut possibilities = self.suggest_correct_spelling(word_chars);

            // If the misspelled word is capitalized, capitalize the results too.
            if let Some(mis_f) = word_chars.first()
                && mis_f.is_uppercase()
            {
                for sug_f in possibilities.iter_mut().filter_map(|w| {
                    // Skip words that have uppercase chars in any position except the first.
                    // (For words with specific capitalization, like 'macOS')
                    w.iter()
                        .skip(1)
                        .all(|c| !c.is_uppercase())
                        .then_some(w.first_mut())
                        .flatten()
                }) {
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
    use std::sync::Arc;

    use super::{LintKind, Linter, SpellCheck};
    use crate::languages::LanguageFamily;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};
    use crate::spell::FstDictionary;
    use crate::{Document, PortugueseDialect};

    // Capitalization tests

    #[test]
    fn brasil_capitalized() {
        assert_suggestion_result(
            "A palavra brasil deveria ser capitalizada.",
            SpellCheck::new(
                FstDictionary::curated_select_language(LanguageFamily::Portuguese),
                PortugueseDialect::Brazilian,
            ),
            "A palavra Brasil deveria ser capitalizada.",
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
            LanguageFamily::Portuguese,
        );
    }

    fn brazilian() -> SpellCheck<Arc<FstDictionary>> {
        SpellCheck::new(
            FstDictionary::curated_select_language(LanguageFamily::Portuguese),
            PortugueseDialect::Brazilian,
        )
    }

    // Correctly spelled text

    #[test]
    fn simple_sentence_is_clean() {
        assert_no_lints(
            "Eu sou feliz com a minha vida.",
            brazilian(),
            LanguageFamily::Portuguese,
        );
    }

    #[test]
    fn accented_words_are_clean() {
        assert_no_lints(
            "Tu estás triste. Eles são o meu mundo.",
            brazilian(),
            LanguageFamily::Portuguese,
        )
    }

    #[test]
    fn sentence_initial_capital_is_clean() {
        assert_no_lints("Ela era feliz.", brazilian(), LanguageFamily::Portuguese);
    }

    #[test]
    fn all_caps_word_is_clean() {
        assert_no_lints("MUNDO", brazilian(), LanguageFamily::Portuguese);
    }

    #[test]
    fn regular_plurals_are_clean() {
        assert_no_lints(
            "Os dias e as noites.",
            brazilian(),
            LanguageFamily::Portuguese,
        );
    }

    #[test]
    fn plurals_ending_in_r_are_clean() {
        assert_no_lints(
            "Os amores e os deveres.",
            brazilian(),
            LanguageFamily::Portuguese,
        );
    }

    #[test]
    fn plurals_ending_in_l_are_clean() {
        assert_no_lints("Tais e quais.", brazilian(), LanguageFamily::Portuguese);
    }

    #[test]
    fn numbers_are_not_flagged() {
        assert_no_lints(
            "Eu fui de 1990 a 2000.",
            brazilian(),
            LanguageFamily::Portuguese,
        );
    }

    #[test]
    fn urls_are_not_flagged() {
        assert_no_lints(
            "Eu fui em https://writewithharper.com.",
            brazilian(),
            LanguageFamily::Portuguese,
        );
    }

    #[test]
    fn european_dialect_accepts_shared_words() {
        assert_no_lints(
            "Eu sou feliz com a minha vida.",
            SpellCheck::new(
                FstDictionary::curated_select_language(LanguageFamily::Portuguese),
                PortugueseDialect::European,
            ),
            LanguageFamily::Portuguese,
        );
    }

    // Misspelled text

    #[test]
    fn corrects_doubled_letter() {
        assert_suggestion_result(
            "Eu sou feliz com a minha vidda.",
            brazilian(),
            "Eu sou feliz com a minha vida.",
            LanguageFamily::Portuguese,
        );
    }

    #[test]
    fn corrects_wrong_letter() {
        assert_suggestion_result(
            "Eu sou felis.",
            brazilian(),
            "Eu sou feliz.",
            LanguageFamily::Portuguese,
        );
    }

    #[test]
    fn restores_missing_acute_accent() {
        assert_suggestion_result(
            "Tu estas triste.",
            brazilian(),
            "Tu estás triste.",
            LanguageFamily::Portuguese,
        );
    }

    #[test]
    fn restores_missing_tilde() {
        assert_suggestion_result(
            "Eles sao o meu mundo.",
            brazilian(),
            "Eles são o meu mundo.",
            LanguageFamily::Portuguese,
        );
    }

    #[test]
    fn keeps_capitalization_of_misspelled_word() {
        assert_suggestion_result("Mundu", brazilian(), "Mundo", LanguageFamily::Portuguese);
    }

    #[test]
    fn flags_each_misspelled_word() {
        assert_lint_count(
            "Eu sou felis com a minha vidda.",
            brazilian(),
            2,
            LanguageFamily::Portuguese,
        );
    }

    #[test]
    fn flags_english_words() {
        assert_lint_count("The world", brazilian(), 2, LanguageFamily::Portuguese);
    }

    #[test]
    fn single_suggestion_message_names_the_word() {
        let document = Document::new_plain_portuguese_curated("Eu sou felis.");
        let lints = brazilian().lint(&document);

        assert_eq!(lints.len(), 1);
        assert_eq!(lints[0].lint_kind, LintKind::Spelling);
        if lints[0].suggestions.len() == 1 {
            assert_eq!(lints[0].message, "Did you mean `feliz`?");
        } else {
            assert_eq!(lints[0].message, "Did you mean to spell `felis` this way?");
        }
    }
}
