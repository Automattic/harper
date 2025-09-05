use std::num::NonZero;

use lru::LruCache;
use serde::{Deserialize, Serialize};
use smallvec::ToSmallVec;

use super::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::document::Document;
use crate::spell::{suggest_correct_spelling, Dictionary};
use crate::{CharString, CharStringExt, Dialect, TokenStringExt};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SpellCheckConfig {
    pub ignore_capitalized: bool,
    pub ignore_abbreviations: bool,
}

pub struct SpellCheck<T>
where
    T: Dictionary,
{
    dictionary: T,
    word_cache: LruCache<CharString, Vec<CharString>>,
    dialect: Dialect,
    config: SpellCheckConfig,
}

impl<T: Dictionary> SpellCheck<T> {
    pub fn new(dictionary: T, dialect: Dialect, config: SpellCheckConfig) -> Self {
        Self {
            dictionary,
            word_cache: LruCache::new(NonZero::new(10000).unwrap()),
            dialect,
            config,
        }
    }

    /// Update the spell check configuration without rebuilding the entire linter
    pub fn update_config(&mut self, config: SpellCheckConfig) {
        self.config = config;
    }

    const MAX_SUGGESTIONS: usize = 3;

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

fn check_abbrv(chars: &[char]) -> bool {
    if chars.is_empty() {
        return false;
    }
    
    if chars.iter().all(|&c| c.is_uppercase() || c.is_numeric()) {
        return true;
    }
    
    // Find valid suffixes at the end
    let mut found_lowercase = false;
    let mut lowercase_start_idx = chars.len();
    

    for (i, &c) in chars.iter().enumerate().rev() {
        if c.is_lowercase() {
            lowercase_start_idx = i;
            found_lowercase = true;
        } else if found_lowercase {
            break;
        }
    }
    
    if !found_lowercase {
        return chars.iter().all(|&c| c.is_uppercase() || c.is_numeric());
    }
    
    let lowercase_part: String = chars[lowercase_start_idx..].iter().collect();
    
    let allowed_suffixes = ["d", "s", "'s", "ed"];
    let is_valid_suffix = allowed_suffixes.iter().any(|&suffix| lowercase_part == suffix);
    
    if !is_valid_suffix {
        return false;
    }
    
    chars[..lowercase_start_idx].iter().all(|&c| c.is_uppercase() || c.is_numeric())
}



impl<T: Dictionary> Linter for SpellCheck<T> {
    fn update_spell_check_config(&mut self, config: SpellCheckConfig) {
        self.config = config;
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word in document.iter_words() {
            let word_chars = document.get_span_content(&word.span);

            if self.config.ignore_capitalized && word_chars.first().is_some_and(|c| c.is_uppercase()) {
                continue;
            }

            if self.config.ignore_abbreviations && check_abbrv(word_chars) {
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
}

#[cfg(test)]
mod tests {
    use super::{SpellCheck, SpellCheckConfig};
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
            SpellCheck::new(FstDictionary::curated(), Dialect::American, SpellCheckConfig::default()),
            "The word America should be capitalized.",
        );
    }

    // Dialect tests

    #[test]
    fn harper_automattic_capitalized() {
        assert_lint_count(
            "So should harper and automattic.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, SpellCheckConfig::default()),
            2,
        );
    }

    #[test]
    fn american_color_in_british_dialect() {
        assert_lint_count(
            "Do you like the color?",
            SpellCheck::new(FstDictionary::curated(), Dialect::British, SpellCheckConfig::default()),
            1,
        );
    }

    #[test]
    fn canadian_words_in_australian_dialect() {
        assert_lint_count(
            "Does your mom like yogourt?",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian, SpellCheckConfig::default()),
            2,
        );
    }

    #[test]
    fn australian_words_in_canadian_dialect() {
        assert_lint_count(
            "We mine bauxite to make aluminium.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Canadian, SpellCheckConfig::default()),
            1,
        );
    }

    #[test]
    fn mum_and_mummy_not_just_commonwealth() {
        assert_lint_count(
            "Mum's the word about that Egyptian mummy.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, SpellCheckConfig::default()),
            0,
        );
    }

    #[test]
    fn australian_verandah() {
        assert_lint_count(
            "Our house has a verandah.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian, SpellCheckConfig::default()),
            0,
        );
    }

    #[test]
    fn australian_verandah_in_american_dialect() {
        assert_lint_count(
            "Our house has a verandah.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, SpellCheckConfig::default()),
            1,
        );
    }

    #[test]
    fn austrlaian_verandah_in_british_dialect() {
        assert_lint_count(
            "Our house has a verandah.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British, SpellCheckConfig::default()),
            1,
        );
    }

    #[test]
    fn australian_verandah_in_canadian_dialect() {
        assert_lint_count(
            "Our house has a verandah.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Canadian, SpellCheckConfig::default()),
            1,
        );
    }

    #[test]
    fn mixing_australian_and_canadian_dialects() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian, SpellCheckConfig::default()),
            1,
        );
    }

    #[test]
    fn mixing_canadian_and_australian_dialects() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Canadian, SpellCheckConfig::default()),
            1,
        );
    }

    #[test]
    fn australian_and_canadian_spellings_that_are_not_american() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, SpellCheckConfig::default()),
            2,
        );
    }

    #[test]
    fn australian_and_canadian_spellings_that_are_not_british() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British, SpellCheckConfig::default()),
            2,
        );
    }

    #[test]
    fn australian_labour_vs_labor() {
        assert_lint_count(
            "In Australia we write 'labour' but the political party is the 'Labor Party'.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian, SpellCheckConfig::default()),
            0,
        )
    }

    #[test]
    fn australian_words_flagged_for_american_english() {
        assert_lint_count(
            "There's an esky full of beers in the back of the ute.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, SpellCheckConfig::default()),
            2,
        );
    }

    #[test]
    fn american_words_not_flagged_for_australian_english() {
        assert_lint_count(
            "In general, utes have unibody construction while pickups have frames.",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian, SpellCheckConfig::default()),
            0,
        );
    }

    #[test]
    fn abandonware_correction() {
        assert_suggestion_result(
            "abanonedware",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian, SpellCheckConfig::default()),
            "abandonware",
        );
    }

    // Unit tests for specific spellcheck corrections

    #[test]
    fn corrects_abandonedware_1131_1166() {
        // assert_suggestion_result(
        assert_top3_suggestion_result(
            "Abandonedware is abandoned. Do not bother submitting issues about the empty page bug. Author moved to greener pastures",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, SpellCheckConfig::default()),
            "Abandonware is abandoned. Do not bother submitting issues about the empty page bug. Author moved to greener pastures",
        );
    }

    #[test]
    fn afterwards_not_us() {
        assert_lint_count(
            "afterwards",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, SpellCheckConfig::default()),
            1,
        );
    }

    #[test]
    fn afterward_is_us() {
        assert_lint_count(
            "afterward",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, SpellCheckConfig::default()),
            0,
        );
    }

    #[test]
    fn afterward_not_au() {
        assert_lint_count(
            "afterward",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian, SpellCheckConfig::default()),
            1,
        );
    }

    #[test]
    fn afterwards_is_au() {
        assert_lint_count(
            "afterwards",
            SpellCheck::new(FstDictionary::curated(), Dialect::Australian, SpellCheckConfig::default()),
            0,
        );
    }

    #[test]
    fn afterward_not_ca() {
        assert_lint_count(
            "afterward",
            SpellCheck::new(FstDictionary::curated(), Dialect::Canadian, SpellCheckConfig::default()),
            1,
        );
    }

    #[test]
    fn afterwards_is_ca() {
        assert_lint_count(
            "afterwards",
            SpellCheck::new(FstDictionary::curated(), Dialect::Canadian, SpellCheckConfig::default()),
            0,
        );
    }

    #[test]
    fn afterward_not_uk() {
        assert_lint_count(
            "afterward",
            SpellCheck::new(FstDictionary::curated(), Dialect::British, SpellCheckConfig::default()),
            1,
        );
    }

    #[test]
    fn afterwards_is_uk() {
        assert_lint_count(
            "afterwards",
            SpellCheck::new(FstDictionary::curated(), Dialect::British, SpellCheckConfig::default()),
            0,
        );
    }

    #[test]
    fn corrects_hes() {
        assert_suggestion_result(
            "hes",
            SpellCheck::new(FstDictionary::curated(), Dialect::British, SpellCheckConfig::default()),
            "he's",
        );
    }

    #[test]
    fn corrects_shes() {
        assert_suggestion_result(
            "shes",
            SpellCheck::new(FstDictionary::curated(), Dialect::British, SpellCheckConfig::default()),
            "she's",
        );
    }
}

#[cfg(test)]
mod conditional {
    use super::{SpellCheck, SpellCheckConfig};
    use crate::spell::FstDictionary;
    use crate::{
        Dialect,
        linting::tests::{
            assert_lint_count
        },
    };

    #[test]
    fn ignore_capitalized_word() {
        let config = SpellCheckConfig {
            ignore_capitalized: true,
            ignore_abbreviations: false,
        };
        assert_lint_count(
            "This is a Tst.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, config),
            0,
        );
    }

    #[test]
    fn dont_ignore_capitalized_word() {
        let config = SpellCheckConfig {
            ignore_capitalized: false,
            ignore_abbreviations: false,
        };
        assert_lint_count(
            "This is a Tst.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, config),
            1,
        );
    }

    #[test]
    fn ignore_abbreviation() {
        let config = SpellCheckConfig {
            ignore_capitalized: false,
            ignore_abbreviations: true,
        };
        assert_lint_count(
            "This is an ABBRV.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, config),
            0,
        );
    }

    #[test]
    fn dont_ignore_abbreviation() {
        let config = SpellCheckConfig {
            ignore_capitalized: false,
            ignore_abbreviations: false,
        };
        assert_lint_count(
            "This is an ABBRV.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American, config),
            1,
        );
    }
}
