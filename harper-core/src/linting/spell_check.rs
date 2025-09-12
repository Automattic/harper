use std::num::NonZero;

use lru::LruCache;
use smallvec::ToSmallVec;

use super::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::document::Document;
use crate::spell::{Dictionary, suggest_correct_spelling};
use crate::{CharString, CharStringExt, Dialect, TokenStringExt};

pub struct SpellCheck<T>
where
    T: Dictionary,
{
    dictionary: T,
    word_cache: LruCache<CharString, Vec<CharString>>,
    dialect: Dialect,
    pub(crate) ignore_all_caps: bool,
}

impl<T: Dictionary> SpellCheck<T> {
    pub fn new(dictionary: T, dialect: Dialect) -> Self {
        Self {
            dictionary,
            word_cache: LruCache::new(NonZero::new(10000).unwrap()),
            dialect,
            ignore_all_caps: false,
        }
    }

    const MAX_SUGGESTIONS: usize = 3;

    fn is_all_caps(&self, word: &[char]) -> bool {
        if word.len() <= 1 {
            return false;
        }

        let word_str: String = word.iter().collect();
        let word_str = word_str.as_str();

        // Check for allowed single suffixes only: 's, 'd, ed, s
        let suffixes = ["'s", "'d", "ed", "s", "es"];

        for suffix in &suffixes {
            if let Some(stem) = word_str.strip_suffix(suffix)
                && !stem.is_empty()
            {
                let stem_chars: Vec<char> = stem.chars().collect();
                // Check if stem is all caps (ignoring non-alphabetic characters)
                if stem_chars
                    .iter()
                    .all(|c| c.is_uppercase() || !c.is_alphabetic())
                {
                    // Make sure the stem doesn't end with another suffix
                    for other_suffix in &suffixes {
                        if stem.ends_with(other_suffix) {
                            return false;
                        }
                    }
                    return true;
                }
            }
        }

        // If no suffix matches, check the whole word
        word.iter().all(|c| c.is_uppercase() || !c.is_alphabetic())
    }

    fn suggest_correct_spelling(&mut self, word: &[char]) -> Vec<CharString> {
        if let Some(hit) = self.word_cache.get(word) {
            hit.clone()
        } else {
            let suggestions = self.uncached_suggest_correct_spelling(word);
            self.word_cache.put(word.into(), suggestions.clone());
            suggestions
        }
    }
    fn uncached_suggest_correct_spelling(&self, word: &[char]) -> Vec<CharString> {
        // Back off until we find a match.
        for dist in 2..5 {
            let suggestions: Vec<CharString> =
                suggest_correct_spelling(word, 100, dist, &self.dictionary)
                    .into_iter()
                    .filter(|v| {
                        // ignore entries outside the configured dialect
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

            // Skip all-caps words if flag is set
            if self.ignore_all_caps && self.is_all_caps(word_chars) {
                continue;
            }

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

            let suggestions = possibilities
                .iter()
                .map(|word| Suggestion::ReplaceWith(word.to_vec()));

            // If there's only one suggestion, save the user a step in the GUI
            let message = if suggestions.len() == 1 {
                format!(
                    "Did you mean `{}`?",
                    possibilities.last().unwrap().iter().collect::<String>()
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

    fn configure_spell_check(&mut self, ignore_all_caps: bool) {
        self.ignore_all_caps = ignore_all_caps;
    }
}

#[cfg(test)]
mod tests {
    use super::SpellCheck;
    use crate::spell::FstDictionary;
    use crate::{
        Dialect,
        linting::tests::{
            assert_lint_count, assert_suggestion_result, assert_top3_suggestion_result,
        },
    };

    // Capitalization tests

    #[test]
    fn america_capitalized() {
        assert_suggestion_result(
            "The word america should be capitalized.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "The word America should be capitalized.",
        );
    }

    // Dialect tests

    #[test]
    fn harper_automattic_capitalized() {
        assert_lint_count(
            "So should harper and automattic.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            2,
        );
    }

    #[test]
    fn american_color_in_british_dialect() {
        assert_lint_count(
            "Do you like the color?",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            1,
        );
    }

    #[test]
    fn canadian_words_in_australian_dialect() {
        assert_lint_count(
            "Does your mom like yogourt?",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian),
            2,
        );
    }

    #[test]
    fn australian_words_in_canadian_dialect() {
        assert_lint_count(
            "We mine bauxite to make aluminium.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Canadian),
            1,
        );
    }

    #[test]
    fn mum_and_mummy_not_just_commonwealth() {
        assert_lint_count(
            "Mum's the word about that Egyptian mummy.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            0,
        );
    }

    #[test]
    fn australian_verandah() {
        assert_lint_count(
            "Our house has a verandah.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian),
            0,
        );
    }

    #[test]
    fn australian_verandah_in_american_dialect() {
        assert_lint_count(
            "Our house has a verandah.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            1,
        );
    }

    #[test]
    fn austrlaian_verandah_in_british_dialect() {
        assert_lint_count(
            "Our house has a verandah.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            1,
        );
    }

    #[test]
    fn australian_verandah_in_canadian_dialect() {
        assert_lint_count(
            "Our house has a verandah.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Canadian),
            1,
        );
    }

    #[test]
    fn mixing_australian_and_canadian_dialects() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian),
            1,
        );
    }

    #[test]
    fn mixing_canadian_and_australian_dialects() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Canadian),
            1,
        );
    }

    #[test]
    fn australian_and_canadian_spellings_that_are_not_american() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            2,
        );
    }

    #[test]
    fn australian_and_canadian_spellings_that_are_not_british() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            2,
        );
    }

    #[test]
    fn australian_labour_vs_labor() {
        assert_lint_count(
            "In Australia we write 'labour' but the political party is the 'Labor Party'.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian),
            0,
        )
    }

    #[test]
    fn australian_words_flagged_for_american_english() {
        assert_lint_count(
            "There's an esky full of beers in the back of the ute.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            2,
        );
    }

    #[test]
    fn american_words_not_flagged_for_australian_english() {
        assert_lint_count(
            "In general, utes have unibody construction while pickups have frames.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian),
            0,
        );
    }

    #[test]
    fn abandonware_correction() {
        assert_suggestion_result(
            "abanonedware",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian),
            "abandonware",
        );
    }

    // Unit tests for specific spellcheck corrections

    #[test]
    fn corrects_abandonedware_1131_1166() {
        // assert_suggestion_result(
        assert_top3_suggestion_result(
            "Abandonedware is abandoned. Do not bother submitting issues about the empty page bug. Author moved to greener pastures",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Abandonware is abandoned. Do not bother submitting issues about the empty page bug. Author moved to greener pastures",
        );
    }

    #[test]
    fn afterwards_not_us() {
        assert_lint_count(
            "afterwards",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            1,
        );
    }

    #[test]
    fn afterward_is_us() {
        assert_lint_count(
            "afterward",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            0,
        );
    }

    #[test]
    fn afterward_not_au() {
        assert_lint_count(
            "afterward",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian),
            1,
        );
    }

    #[test]
    fn afterwards_is_au() {
        assert_lint_count(
            "afterwards",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian),
            0,
        );
    }

    #[test]
    fn afterward_not_ca() {
        assert_lint_count(
            "afterward",
            SpellCheck::new(FstDictionary::curated(), Dialect::Canadian),
            1,
        );
    }

    #[test]
    fn afterwards_is_ca() {
        assert_lint_count(
            "afterwards",
            SpellCheck::new(FstDictionary::curated(), Dialect::Canadian),
            0,
        );
    }

    #[test]
    fn afterward_not_uk() {
        assert_lint_count(
            "afterward",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            1,
        );
    }

    #[test]
    fn afterwards_is_uk() {
        assert_lint_count(
            "afterwards",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            0,
        );
    }

    #[test]
    fn ignore_all_caps_with_suffixes() {
        let mut spell_check = SpellCheck::new(FstDictionary::curated(), Dialect::American);
        spell_check.ignore_all_caps = true;

        // These should be ignored (all caps + allowed suffixes)
        assert_lint_count("APIs", spell_check, 0);
        spell_check = SpellCheck::new(FstDictionary::curated(), Dialect::American);
        spell_check.ignore_all_caps = true;

        assert_lint_count("CPU's", spell_check, 0);
        spell_check = SpellCheck::new(FstDictionary::curated(), Dialect::American);
        spell_check.ignore_all_caps = true;

        assert_lint_count("API'd", spell_check, 0);
    }

    #[test]
    fn dont_ignore_suffix_combinations() {
        let mut spell_check = SpellCheck::new(FstDictionary::curated(), Dialect::American);
        spell_check.ignore_all_caps = true;

        // Should NOT be ignored (combination of suffixes)
        assert_lint_count("CPUsed", spell_check, 1);
    }

    #[test]
    fn ignore_all_caps_basic() {
        let mut spell_check = SpellCheck::new(FstDictionary::curated(), Dialect::American);
        spell_check.ignore_all_caps = true;

        // Should be ignored (all caps)
        assert_lint_count("API", spell_check, 0);
        spell_check = SpellCheck::new(FstDictionary::curated(), Dialect::American);
        spell_check.ignore_all_caps = true;

        assert_lint_count("CPU", spell_check, 0);
    }

    #[test]
    fn dont_ignore_mixed_case() {
        let mut spell_check = SpellCheck::new(FstDictionary::curated(), Dialect::American);
        spell_check.ignore_all_caps = true;

        // Should still lint mixed case misspellings
        assert_lint_count("speling", spell_check, 1);
    }

    #[test]
    fn corrects_hes() {
        assert_suggestion_result(
            "hes",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "he's",
        );
    }

    #[test]
    fn corrects_shes() {
        assert_suggestion_result(
            "shes",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "she's",
        );
    }
}
