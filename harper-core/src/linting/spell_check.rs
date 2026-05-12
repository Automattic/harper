use std::num::NonZero;

use lru::LruCache;
use smallvec::ToSmallVec;

use crate::{
    document::Document,
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::{Dictionary, suggest_correct_spelling},
    {CharString, CharStringExt, Dialect, TokenStringExt},
};

pub struct SpellCheck<T>
where
    T: Dictionary,
{
    dict: T,
    suggestion_cache: LruCache<CharString, Vec<CharString>>,
    dialect: Dialect,
}

impl<T: Dictionary> SpellCheck<T> {
    pub fn new(dictionary: T, dialect: Dialect) -> Self {
        Self {
            dict: dictionary,
            suggestion_cache: LruCache::new(NonZero::new(10000).unwrap()),
            dialect,
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
                suggest_correct_spelling(word, 200, dist, &self.dict)
                    .into_iter()
                    .filter(|v| {
                        // Ignore entries outside the configured dialect
                        self.dict
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

        let mut skip_past: Option<usize> = None;
        for word_idx in document.iter_word_indices() {
            // Skip words that already passed spellcheck as part of a compound with the previous word.
            if let Some(skip_past) = skip_past
                && word_idx <= skip_past
            {
                continue;
            }
            let word = document.get_token(word_idx).unwrap();
            let word_chars = document.get_span_content(&word.span);

            // Is the word in the dictionary on its own?
            if let Some(Some(metadata)) = word.kind.as_word()
                && metadata.dialects.is_dialect_enabled(self.dialect)
                && (self.dict.contains_exact_word(word_chars)
                    || self.dict.contains_exact_word(&word_chars.to_lower()))
            {
                continue;
            };

            // Check if this word forms a compound with the next word
            if let (Some(next_tok), Some(next_next_tok)) = (
                document.get_token_offset(word_idx, 1),
                document.get_token_offset(word_idx, 2),
            ) && (next_tok.kind.is_whitespace() || next_tok.kind.is_hyphen())
                && next_next_tok.kind.is_word()
            {
                let next_word_chars = document.get_span_content(&next_next_tok.span);
                let mut compound_chars =
                    Vec::with_capacity(word_chars.len() + 1 + next_word_chars.len());
                compound_chars.extend_from_slice(word_chars);
                compound_chars.push(if next_tok.kind.is_hyphen() { '-' } else { ' ' });
                compound_chars.extend_from_slice(next_word_chars);

                if self.dict.contains_exact_word(&compound_chars) {
                    skip_past = Some(word_idx + 2);
                    continue;
                }
            };

            // Also check if this word forms a compound with the previous word
            if let (Some(prev_tok), Some(prev_prev_tok)) = (
                document.get_token_offset(word_idx, -1),
                document.get_token_offset(word_idx, -2),
            ) && (prev_tok.kind.is_whitespace() || prev_tok.kind.is_hyphen())
                && prev_prev_tok.kind.is_word()
            {
                let prev_word_chars = document.get_span_content(&prev_prev_tok.span);
                let mut compound_chars =
                    Vec::with_capacity(prev_word_chars.len() + 1 + word_chars.len());
                compound_chars.extend_from_slice(prev_word_chars);
                compound_chars.push(if prev_tok.kind.is_hyphen() { '-' } else { ' ' });
                compound_chars.extend_from_slice(word_chars);

                if self.dict.contains_exact_word(&compound_chars) {
                    continue;
                }
            };

            let mut candidates = self.suggest_correct_spelling(word_chars);

            // If the misspelled word is in title case, apply title case to suggestions too.
            if let Some(err_1st_char) = word_chars.first()
                && err_1st_char.is_uppercase()
            {
                for cand_1st_char in candidates.iter_mut().filter_map(|sugg_word| {
                    // Only process suggestions that don't have "exotic casing", thus preserving
                    // words with modern capitalization patterns like 'macOS', 'iPhone', etc.
                    let has_exotic_casing = sugg_word.iter().skip(1).any(|c| c.is_uppercase());

                    if !has_exotic_casing {
                        sugg_word.first_mut()
                    } else {
                        None
                    }
                }) {
                    *cand_1st_char = cand_1st_char.to_uppercase().next().unwrap();
                }
            }

            let suggestions: Vec<_> = candidates
                .iter()
                .map(|sug| Suggestion::ReplaceWith(sug.to_vec()))
                .collect();

            // If there's only one suggestion, save the user a step in the GUI
            let message = if suggestions.len() == 1 {
                format!(
                    "Did you mean `{}`?",
                    candidates.first().unwrap().iter().collect::<String>()
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
mod tests {
    use strum::IntoEnumIterator;

    use super::SpellCheck;
    use crate::{
        Dialect,
        dict_word_metadata::DialectFlags,
        linting::{
            Linter,
            tests::{
                assert_good_and_bad_suggestions, assert_lint_count, assert_no_lints,
                assert_suggestion_result,
            },
        },
        spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary},
        {DictWordMetadata, Document},
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
    fn australian_verandah_in_british_dialect() {
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
        assert_suggestion_result(
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

    #[test]
    fn issue_1876() {
        let user_dialect = Dialect::American;

        // Create a user dictionary with a word normally of another dialect in it.
        let mut user_dict = MutableDictionary::new();
        user_dict.append_word_str(
            "Calibre",
            DictWordMetadata {
                dialects: DialectFlags::from_dialect(user_dialect),
                ..Default::default()
            },
        );

        // Create a merged dictionary, using curated first.
        let mut merged_dict = MergedDictionary::new();
        merged_dict.add_dictionary(FstDictionary::curated());
        merged_dict.add_dictionary(std::sync::Arc::from(user_dict));
        assert!(merged_dict.contains_word_str("Calibre"));

        // No dialect issues should be found if the word from another dialect is in our user dictionary.
        assert_eq!(
            SpellCheck::new(merged_dict.clone(), user_dialect)
                .lint(&Document::new_markdown_default(
                    "I like to use the software Calibre.",
                    &merged_dict
                ))
                .len(),
            0,
            "Calibre is not part of the user's dialect!"
        );

        assert_eq!(
            SpellCheck::new(merged_dict.clone(), user_dialect)
                .lint(&Document::new_markdown_default(
                    "I like to use the spelling colour.",
                    &merged_dict
                ))
                .len(),
            1
        );
    }

    #[test]
    fn matt_is_allowed() {
        for dialect in Dialect::iter() {
            dbg!(dialect);
            assert_no_lints(
                "Matt is a great name.",
                SpellCheck::new(FstDictionary::curated(), dialect),
            );
        }
    }

    #[test]
    fn issue_2026() {
        assert_suggestion_result(
            "'Tere' is supposed to be 'There'",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "'There' is supposed to be 'There'",
        );

        assert_suggestion_result(
            "'fll' is supposed to be 'fill'",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "'fill' is supposed to be 'fill'",
        );
    }
    #[test]
    fn issue_2261() {
        assert_suggestion_result(
            "Generaly",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Generally",
        );
    }

    #[test]
    fn flag_prepone_in_non_indian_english() {
        assert_lint_count(
            "We had to prepone the meeting",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            1,
        );
    }

    #[test]
    fn dont_flag_prepone_in_indian_english() {
        assert_no_lints(
            "We had to prepone the meeting",
            SpellCheck::new(FstDictionary::curated(), Dialect::Indian),
        );
    }

    #[test]
    fn dont_flag_pr() {
        assert_no_lints(
            "PR",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
        );
    }

    #[test]
    fn no_improper_suggestion_for_macos() {
        assert_good_and_bad_suggestions(
            "MacOS",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            &["macOS"],
            &["MacOS"],
        );
    }

    // Tests that were previously in `spell/mod.rs`

    // Tests for dialect-specific misspelling patterns

    // is_ou_misspelling
    #[test]
    fn suggest_color_for_colour_lowercase() {
        assert_suggestion_result(
            "colour",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "color",
        );
    }

    #[test]
    fn suggest_colour_for_color_lowercase() {
        assert_suggestion_result(
            "color",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "colour",
        );
    }

    // titlecase
    #[test]
    fn suggest_color_for_colour_titlecase() {
        assert_suggestion_result(
            "Colour",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Color",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_colour_for_color_titlecase() {
        assert_suggestion_result(
            "Color",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Colour",
        );
    }

    // all-caps
    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_color_for_colour_all_caps() {
        assert_suggestion_result(
            "COLOUR",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "COLOR",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_colour_for_color_all_caps() {
        assert_suggestion_result(
            "COLOR",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "COLOUR",
        );
    }

    // is_cksz_misspelling

    // s/z as in realise/realize
    #[test]
    #[ignore = "both spellings are acceptable in UK, AU, and IN despite popular opinion"]
    fn suggest_realise_for_realize() {
        assert_suggestion_result(
            "realize",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "realise",
        );
    }

    #[test]
    fn suggest_realize_for_realise() {
        assert_suggestion_result(
            "realise",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "realize",
        );
    }

    #[test]
    #[ignore = "both spellings are acceptable in UK, AU, and IN despite popular opinion"]
    fn suggest_realise_for_realize_titlecase() {
        assert_suggestion_result(
            "Realize",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Realise",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_realize_for_realise_titlecase() {
        assert_suggestion_result(
            "Realise",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Realize",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_realise_for_realize_all_caps() {
        assert_suggestion_result(
            "REALIZE",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "REALISE",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_realize_for_realise_all_caps() {
        assert_suggestion_result(
            "REALISE",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "REALIZE",
        );
    }

    // s/c as in defense/defence
    #[test]
    fn suggest_defence_for_defense() {
        assert_suggestion_result(
            "defense",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "defence",
        );
    }

    #[test]
    fn suggest_defense_for_defence() {
        assert_suggestion_result(
            "defence",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "defense",
        );
    }

    #[test]
    fn suggest_defense_for_defence_titlecase() {
        assert_suggestion_result(
            "Defense",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Defence",
        );
    }

    #[test]
    fn suggest_defence_for_defense_titlecase() {
        assert_suggestion_result(
            "Defence",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Defense",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_defense_for_defence_all_caps() {
        assert_suggestion_result(
            "DEFENSE",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "DEFENCE",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_defence_for_defense_all_caps() {
        assert_suggestion_result(
            "DEFENCE",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "DEFENSE",
        );
    }

    // k/c as in skeptic/sceptic
    #[test]
    fn suggest_sceptic_for_skeptic() {
        assert_suggestion_result(
            "skeptic",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "sceptic",
        );
    }

    #[test]
    fn suggest_skeptic_for_sceptic() {
        assert_suggestion_result(
            "sceptic",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "skeptic",
        );
    }

    #[test]
    fn suggest_sceptic_for_skeptic_titlecase() {
        assert_suggestion_result(
            "Skeptic",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Sceptic",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_skeptic_for_sceptic_titlecase() {
        assert_suggestion_result(
            "Sceptic",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Skeptic",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_skeptic_for_sceptic_all_caps() {
        assert_suggestion_result(
            "SKEPTIC",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "SCEPTIC",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_sceptic_for_skeptic_all_caps() {
        assert_suggestion_result(
            "SCEPTIC",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "SKEPTIC",
        );
    }

    // is_er_misspelling
    // as in meter/metre
    #[test]
    fn suggest_centimeter_for_centimetre() {
        assert_suggestion_result(
            "centimetre",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "centimeter",
        );
    }

    #[test]
    fn suggest_centimetre_for_centimeter() {
        assert_suggestion_result(
            "centimeter",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "centimetre",
        );
    }

    #[test]
    fn suggest_centimeter_for_centimetre_titlecase() {
        assert_suggestion_result(
            "Centimetre",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Centimeter",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_centimetre_for_centimeter_titlecase() {
        assert_suggestion_result(
            "Centimeter",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Centimetre",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_centimeter_for_centimetre_all_caps() {
        assert_suggestion_result(
            "CENTIMETRE",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "CENTIMETER",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_centimetre_for_centimeter_all_caps() {
        assert_suggestion_result(
            "CENTIMETER",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "CENTIMETRE",
        );
    }

    // is_ll_misspelling
    // as in traveller/traveler
    #[test]
    fn suggest_traveler_for_traveller() {
        assert_suggestion_result(
            "traveller",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "traveler",
        );
    }

    #[test]
    fn suggest_traveller_for_traveler() {
        assert_suggestion_result(
            "traveler",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "traveller",
        );
    }

    #[test]
    fn suggest_traveler_for_traveller_titlecase() {
        assert_suggestion_result(
            "Traveller",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "Traveler",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_traveller_for_traveler_titlecase() {
        assert_suggestion_result(
            "Traveler",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "Traveller",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_traveler_for_traveller_all_caps() {
        assert_suggestion_result(
            "TRAVELLER",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "TRAVELER",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_traveller_for_traveler_all_caps() {
        assert_suggestion_result(
            "TRAVELER",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "TRAVELLER",
        );
    }

    // is_ay_ey_misspelling
    // as in gray/grey

    #[test]
    fn suggest_grey_for_gray_in_non_american() {
        assert_suggestion_result(
            "I've got a gray cat.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "I've got a grey cat.",
        );
    }

    #[test]
    fn suggest_gray_for_grey_in_american() {
        assert_suggestion_result(
            "It's a greyscale photo.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "It's a grayscale photo.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_grey_for_gray_in_non_american_titlecase() {
        assert_suggestion_result(
            "I've Got a Gray Cat.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "I've Got a Grey Cat.",
        );
    }

    #[test]
    fn suggest_gray_for_grey_in_american_titlecase() {
        assert_suggestion_result(
            "It's a Greyscale Photo.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "It's a Grayscale Photo.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_grey_for_gray_in_non_american_all_caps() {
        assert_suggestion_result(
            "GRAY",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "GREY",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_gray_for_grey_in_american_all_caps() {
        assert_suggestion_result(
            "GREY",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            "GRAY",
        );
    }

    // Tests for non-dialectal misspelling patterns

    // is_ei_ie_misspelling
    #[test]
    fn fix_cheif_and_recieved() {
        assert_suggestion_result(
            "The cheif recieved a letter.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "The chief received a letter.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn fix_cheif_and_recieved_titlecase() {
        assert_suggestion_result(
            "The Cheif Recieved a Letter.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "The Chief Received a Letter.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn fix_cheif_and_recieved_all_caps() {
        assert_suggestion_result(
            "THE CHEIF RECIEVED A LETTER.",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "THE CHEIF RECEIVED A LETTER.",
        );
    }

    #[test]
    fn fix_vs_apostrophe() {
        assert_suggestion_result(
            "v's",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "vs",
        );
    }

    #[test]
    fn fix_vs_typographical_apostrophe() {
        assert_suggestion_result(
            "v’s",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "vs",
        );
    }

    #[test]
    fn fix_childrens_missing_apostrophe() {
        assert_suggestion_result(
            "childrens",
            SpellCheck::new(FstDictionary::curated(), Dialect::British),
            "children's",
        );
    }

    #[test]
    fn hyphenated_compound_1st_word_not_in_dict() {
        assert_lint_count(
            "Greco",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            1,
        );
        assert_no_lints(
            "Greco-Roman",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
        );
    }

    #[test]
    fn hyphenated_compound_2nd_word_not_in_dict() {
        assert_lint_count(
            "droppingly",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            1,
        );
        assert_no_lints(
            "jaw-droppingly",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
        );
    }

    #[test]
    fn hyphenated_compound_neither_word_in_dict() {
        assert_lint_count(
            "goosey. loosey.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            2,
        );
        assert_no_lints(
            "loosey-goosey",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
        );
    }

    #[test]
    fn open_compound_1st_word_not_in_dict() {
        assert_lint_count(
            "welch/plug",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            1,
        );
        assert_no_lints(
            "welch plug",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
        );
    }

    #[test]
    fn open_compound_neither_word_in_dict() {
        assert_lint_count(
            "Holy and holey are words, but moly and moley are not.",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
            2,
        );
        assert_no_lints(
            "holy moly vs holy moley",
            SpellCheck::new(FstDictionary::curated(), Dialect::American),
        );
    }
}
