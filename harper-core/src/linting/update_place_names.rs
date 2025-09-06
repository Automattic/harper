use crate::expr::{Expr, FixedPhrase, LongestMatchOf};
use crate::linting::{ExprLinter, LintKind, Suggestion};
use crate::{Lint, Token, TokenStringExt};

type PlaceNameMappings<'a> = &'a [((i16, &'a str), &'a [&'a str])];

pub struct UpdatePlaceNames<'a> {
    expr: Box<dyn Expr>,
    place_name_mappings: PlaceNameMappings<'a>,
}

impl<'a> Default for UpdatePlaceNames<'a> {
    fn default() -> Self {
        let place_name_mappings: PlaceNameMappings<'a> = &[
            // // Africa
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

        let expr = LongestMatchOf::new(
            place_name_mappings
                .iter()
                .flat_map(|(_, old_names)| old_names.iter())
                .map(|old_name| Box::new(FixedPhrase::from_phrase(old_name)) as Box<dyn Expr>)
                .collect(),
        );

        Self::new(Box::new(expr), place_name_mappings)
    }
}

impl<'a> UpdatePlaceNames<'a> {
    pub fn new(expr: Box<dyn Expr>, place_name_mappings: PlaceNameMappings<'a>) -> Self {
        Self {
            expr,
            place_name_mappings,
        }
    }
}

impl<'a> ExprLinter for UpdatePlaceNames<'a> {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let old_name = toks.span()?.get_content_string(src);
        let (year, new_name) =
            self.place_name_mappings
                .iter()
                .find_map(|((year, new_name), old_names)| {
                    old_names
                        .iter()
                        .any(|n| n == &old_name)
                        .then_some((year, *new_name))
                })?;

        let suggestions = vec![Suggestion::ReplaceWith(new_name.chars().collect())];

        let message = match year {
            1.. => format!("This place has been officially known as '{new_name}' since {year}"),
            _ => format!("This place is now officially known as '{new_name}'"),
        };

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message,
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "This rule looks for deprecated place names and offers to update them."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{
        tests::{assert_lint_count, assert_suggestion_result},
        update_place_names::UpdatePlaceNames,
    };

    #[test]
    fn update_single_word_name_alone() {
        assert_suggestion_result("Bombay", UpdatePlaceNames::default(), "Mumbai");
    }

    #[test]
    fn update_single_word_name_after_space() {
        assert_suggestion_result(" Bombay", UpdatePlaceNames::default(), " Mumbai");
    }

    #[test]
    fn update_single_word_name_after_punctuation() {
        assert_suggestion_result(";Bombay", UpdatePlaceNames::default(), ";Mumbai");
    }

    #[test]
    fn update_two_word_name_to_single_word_alone() {
        assert_suggestion_result("Ayers Rock", UpdatePlaceNames::default(), "Uluru");
    }

    #[test]
    fn update_two_word_name_to_single_word_after_space() {
        assert_suggestion_result(" Ayers Rock", UpdatePlaceNames::default(), " Uluru");
    }

    #[test]
    fn update_two_word_name_to_single_word_after_punctuation() {
        assert_suggestion_result(";Ayers Rock", UpdatePlaceNames::default(), ";Uluru");
    }

    #[test]
    fn update_single_word_name_to_multi_word_name_alone() {
        assert_suggestion_result("Saigon", UpdatePlaceNames::default(), "Ho Chi Minh City");
    }

    #[test]
    fn update_two_word_name_to_two_word_name_alone() {
        assert_suggestion_result("The Olgas", UpdatePlaceNames::default(), "Kata Tjuta");
    }

    #[test]
    fn dont_flag_multiword_name_with_non_space() {
        assert_lint_count("The, Olgas", UpdatePlaceNames::default(), 0);
    }

    #[test]
    fn dont_flag_multiword_name_with_hyphen() {
        assert_lint_count("The-Olgas", UpdatePlaceNames::default(), 0);
    }

    // TODO: when both old and new names contain whitespace we don't copy the whitespace
    #[test]
    #[ignore = "tabs not supported as whitespace?"]
    fn flag_multiword_name_with_tabs() {
        assert_lint_count("The\tOlgas", UpdatePlaceNames::default(), 1);
    }

    // TODO: when both old and new names contain whitespace we don't copy the whitespace
    #[test]
    #[ignore = "newlines not supported as whitespace?"]
    fn flag_multiword_name_with_newline() {
        assert_lint_count("The\nOlgas", UpdatePlaceNames::default(), 1);
    }

    #[test]
    fn update_two_word_name_to_single_word_at_end_of_sentence() {
        assert_suggestion_result(
            "It's dangerous to climb Ayers Rock.",
            UpdatePlaceNames::default(),
            "It's dangerous to climb Uluru.",
        );
    }

    #[test]
    fn update_two_word_name_to_single_word_at_start_of_sentence() {
        assert_suggestion_result(
            "Ayers Rock is dangerous to climb.",
            UpdatePlaceNames::default(),
            "Uluru is dangerous to climb.",
        );
    }

    #[test]
    fn update_first_old_name() {
        assert_suggestion_result("Leningrad", UpdatePlaceNames::default(), "Saint Petersburg");
    }

    #[test]
    fn update_second_old_name() {
        assert_suggestion_result(
            "Have you ever been to Petrograd before?",
            UpdatePlaceNames::default(),
            "Have you ever been to Saint Petersburg before?",
        );
    }

    #[test]
    fn update_two_word_name_with_two_word_name() {
        assert_suggestion_result(
            "Upper Volta is in Africa.",
            UpdatePlaceNames::default(),
            "Burkina Faso is in Africa.",
        )
    }

    // NOTE: Can't handle place names with obligatory or compulsory "The" perfectly.
    #[test]
    fn update_to_name_with_punctuation() {
        assert_suggestion_result(
            "I've never been to Ivory Coast.",
            UpdatePlaceNames::default(),
            "I've never been to Côte d'Ivoire.",
        )
    }
}
