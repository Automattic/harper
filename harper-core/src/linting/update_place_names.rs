use super::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, Span, TokenStringExt};
use lazy_static::lazy_static;

/// Detect deprecated place names and offer to update them.
#[derive(Debug, Clone, Copy, Default)]
pub struct UpdatePlaceNames;

type PlaceNameEntryStr<'a> = ((i16, &'a str), &'a [&'a str]);
type PlaceNameEntryVecChar = ((i16, Vec<char>), Vec<Vec<Vec<char>>>);

// (year, new name), [old names]
const RAW_PLACE_NAME_UPDATES: &[PlaceNameEntryStr] = &[
    // Africa
    ((1984, "Burkina Faso"), &["Upper Volta"]),
    ((1985, "Côte d'Ivoire"), &["Ivory Coast"]), // TODO: Can we recommend Cote d'Ivoire as well?
    ((2018, "Eswatini"), &["Swaziland"]),
    // ((1995, "Janjanbureh"), &["Georgetown"]), // Too many places named Georgetown / George Town
    // Australia
    ((1993, "Kata Tjuta"), &["The Olgas"]), // TODO: Can we recommend the spelling with the underscore letter(s) as well?
    ((1993, "Uluru"), &["Ayers Rock"]), // TODO: Can we recommend the spelling with the underscore letter as well?
    // Central Asia
    ((1961, "Dushanbe"), &["Stalinabad"]),
    // East Asia
    ((1979, "Beijing"), &["Peking"]),
    ((0, "Guangzhou"), &["Canton"]),
    ((1945, "Taiwan"), &["Formosa"]),
    ((1991, "Ulaanbaatar"), &["Ulan Bator"]),
    // Europe (and nearby)
    ((2016, "Czechia"), &["Czech Republic"]),
    ((1945, "Gdańsk"), &["Danzig"]), // TODO: Can we recommend Gdansk as well?
    ((1992, "Podgorica"), &["Titograd"]),
    ((1936, "Tbilisi"), &["Tiflis"]),
    ((2022, "Türkiye"), &["Turkey"]), // TODO: Can we recommend Turkiye as well?
    // India
    ((2006, "Bengaluru"), &["Bangalore"]),
    ((1996, "Chennai"), &["Madras"]),
    ((2007, "Kochi"), &["Cochin"]),
    ((2001, "Kolkata"), &["Calcutta"]),
    ((1995, "Mumbai"), &["Bombay"]),
    ((2014, "Mysuru"), &["Mysore"]),
    ((2006, "Puducherry"), &["Pondicherry"]),
    ((1978, "Pune"), &["Poona"]),
    ((1991, "Thiruvananthapuram"), &["Trivandrum"]),
    // Latin America
    ((2013, "CDMX"), &["DF"]),
    // Pacific Island nations
    ((1997, "Samoa"), &["Western Samoa"]),
    ((1980, "Vanuatu"), &["New Hebrides"]),
    // Russia
    ((1946, "Kaliningrad"), &["Königsberg"]), // TODO: can we handle Konigsberg and Koenigsberg?
    ((1991, "Saint Petersburg"), &["Leningrad", "Petrograd"]), // TODO: can we add St. Petersburg?
    ((1961, "Volgograd"), &["Stalingrad"]),
    // South Asia
    ((2000, "Busan"), &["Pusan"]),
    ((2018, "Chattogram"), &["Chittagong"]),
    ((1982, "Dhaka"), &["Dacca"]),
    ((1972, "Sri Lanka"), &["Ceylon"]),
    // Southeast Asia
    ((1989, "Cambodia"), &["Kampuchea"]),
    ((1976, "Ho Chi Minh City"), &["Saigon"]),
    ((2017, "Melaka"), &["Malacca"]),
    ((1989, "Myanmar"), &["Burma"]),
    ((1939, "Thailand"), &["Siam"]),
    ((2002, "Timor-Leste"), &["East Timor"]),
    ((1989, "Yangon"), &["Rangoon"]),
    // Ukraine
    ((1992, "Kharkiv"), &["Kharkov"]),
    ((1992, "Kyiv"), &["Kiev"]),
    ((1992, "Luhansk"), &["Lugansk"]),
    ((1992, "Lviv"), &["Lvov"]),
    ((1992, "Odesa"), &["Odessa"]),
    ((1992, "Vinnytsia"), &["Vinnitsa"]),
    ((1992, "Zaporizhzhia"), &["Zaporozhye"]),
];

lazy_static! {
    static ref PLACE_NAME_UPDATES: Vec<PlaceNameEntryVecChar> = {
        RAW_PLACE_NAME_UPDATES
            .iter()
            .map(|(year_and_new_name, old_names)| {
                let year = year_and_new_name.0;
                let new_name = year_and_new_name.1;
                let new_name_vec: Vec<char> = new_name.chars().collect();
                let old_names_as_char_vecs = old_names
                    .iter()
                    .map(|s| s.split(' ').map(|word| word.chars().collect()).collect())
                    .collect();

                // TODO: Generate accentless versions of names with diacritics.
                ((year, new_name_vec), old_names_as_char_vecs)
            })
            .collect()
    };
}

fn get_place_name_updates() -> &'static Vec<PlaceNameEntryVecChar> {
    &PLACE_NAME_UPDATES
}

impl Linter for UpdatePlaceNames {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let place_name_updates = get_place_name_updates();
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            for start_idx in 0..chunk.len() {
                let mut matched_tokens = Vec::new();

                'next_place: for place_name_update in place_name_updates {
                    let place = place_name_update;
                    let old_names = &place.1;

                    'next_old_name: for old_name in old_names {
                        let token_txt = document.get_span_content(&chunk[start_idx].span);
                        let first_word = &old_name[0];

                        if token_txt != first_word {
                            continue 'next_old_name;
                        }

                        let mut token_idx = start_idx;
                        matched_tokens.push(token_idx);

                        for old_name_word in old_name.iter().skip(1) {
                            token_idx += 2;

                            if token_idx >= chunk.len() {
                                matched_tokens.clear();
                                break 'next_old_name;
                            }

                            if !chunk[token_idx - 1].kind.is_whitespace() {
                                matched_tokens.clear();
                                break 'next_old_name;
                            }

                            let next_token_txt = document.get_span_content(&chunk[token_idx].span);
                            if next_token_txt != old_name_word {
                                matched_tokens.clear();
                                break 'next_old_name;
                            }

                            matched_tokens.push(token_idx);
                        }

                        if matched_tokens.len() == old_name.len() {
                            let update = place_name_update;
                            let year = update.0.0;
                            let new_name = &update.0.1;

                            let span_start = chunk[matched_tokens[0]].span.start;
                            let span_end = chunk[*matched_tokens.last().unwrap()].span.end;

                            let message = match year {
                                0 => "A newer name is now in use.".to_string(),
                                _ => format!("A newer name has in use since {}.", year),
                            };

                            lints.push(Lint {
                                span: Span::new(span_start, span_end),
                                lint_kind: LintKind::Style,
                                suggestions: vec![Suggestion::ReplaceWith(new_name.to_vec())],
                                message,
                                ..Default::default()
                            });

                            break 'next_place;
                        } else {
                            unreachable!();
                        }
                    }
                }
            }
        }
        lints
    }

    fn description(&self) -> &str {
        "This rule looks for deprecated place names and offers to update them."
    }
}

#[cfg(test)]
mod tests {
    use super::UpdatePlaceNames;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn update_single_word_name_alone() {
        assert_suggestion_result("Bombay", UpdatePlaceNames, "Mumbai");
    }

    #[test]
    fn update_single_word_name_after_space() {
        assert_suggestion_result(" Bombay", UpdatePlaceNames, " Mumbai");
    }

    #[test]
    fn update_single_word_name_after_punctuation() {
        assert_suggestion_result(";Bombay", UpdatePlaceNames, ";Mumbai");
    }

    #[test]
    fn update_two_word_name_to_single_word_alone() {
        assert_suggestion_result("Ayers Rock", UpdatePlaceNames, "Uluru");
    }

    #[test]
    fn update_two_word_name_to_single_word_after_space() {
        assert_suggestion_result(" Ayers Rock", UpdatePlaceNames, " Uluru");
    }

    #[test]
    fn update_two_word_name_to_single_word_after_punctuation() {
        assert_suggestion_result(";Ayers Rock", UpdatePlaceNames, ";Uluru");
    }

    #[test]
    fn update_single_word_name_to_multi_word_name_alone() {
        assert_suggestion_result("Saigon", UpdatePlaceNames, "Ho Chi Minh City");
    }

    #[test]
    fn update_two_word_name_to_two_word_name_alone() {
        assert_suggestion_result("The Olgas", UpdatePlaceNames, "Kata Tjuta");
    }

    #[test]
    fn dont_flag_multiword_name_with_non_space() {
        assert_lint_count("The, Olgas", UpdatePlaceNames, 0);
    }

    #[test]
    fn dont_flag_multiword_name_with_hyphen() {
        assert_lint_count("The-Olgas", UpdatePlaceNames, 0);
    }

    // TODO: when both old and new names contain whitespace we don't copy the whitespace
    #[test]
    fn flag_multiword_name_with_tabs() {
        assert_lint_count("The\tOlgas", UpdatePlaceNames, 1);
    }

    // TODO: when both old and new names contain whitespace we don't copy the whitespace
    #[test]
    fn flag_multiword_name_with_newline() {
        assert_lint_count("The\nOlgas", UpdatePlaceNames, 1);
    }

    #[test]
    fn update_two_word_name_to_single_word_at_end_of_sentence() {
        assert_suggestion_result(
            "It's dangerous to climb Ayers Rock.",
            UpdatePlaceNames,
            "It's dangerous to climb Uluru.",
        );
    }

    #[test]
    fn update_two_word_name_to_single_word_at_start_of_sentence() {
        assert_suggestion_result(
            "Ayers Rock is dangerous to climb.",
            UpdatePlaceNames,
            "Uluru is dangerous to climb.",
        );
    }

    #[test]
    fn update_first_old_name() {
        assert_suggestion_result("Leningrad", UpdatePlaceNames, "Saint Petersburg");
    }

    #[test]
    fn update_second_old_name() {
        assert_suggestion_result(
            "Have you ever been to Petrograd before?",
            UpdatePlaceNames,
            "Have you ever been to Saint Petersburg before?",
        );
    }

    #[test]
    fn update_two_word_name_with_two_word_name() {
        assert_suggestion_result(
            "Upper Volta is in Africa.",
            UpdatePlaceNames,
            "Burkina Faso is in Africa.",
        )
    }

    // NOTE: Can't handle place names with obligatory or compulsory "The" perfectly.
    #[test]
    fn update_to_name_with_punctuation() {
        assert_suggestion_result(
            "I've never been to Ivory Coast.",
            UpdatePlaceNames,
            "I've never been to Côte d'Ivoire.",
        )
    }
}
