use super::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, Span, TokenStringExt};

/// Detect deprecated place names and offer to update them.
#[derive(Debug, Clone, Copy, Default)]
pub struct UpdatePlaceNames;

use std::collections::HashMap;

const PLACE_NAME_UPDATES: &[((i16, &str), &[&str])] = &[
    // Australia
    ((1993, "Kata Tjuta"), &["The Olgas"]),
    ((1993, "Uluru"), &["Ayers Rock"]),
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
    // Southeast Asia
    ((1989, "Cambodia"), &["Kampuchea"]),
    ((1976, "Ho Chi Minh City"), &["Saigon"]),
    ((1989, "Myanmar"), &["Burma"]),
    ((1939, "Thailand"), &["Siam"]),
    ((2002, "Timor-Leste"), &["East Timor"]),
    // Europe (and nearby)
    ((2016, "Czechia"), &["Czech Republic"]),
    ((1936, "Tbilisi"), &["Tiflis"]),
    ((2022, "Türkiye"), &["Turkey"]),
    // Africa
    ((1984, "Burkina Faso"), &["Upper Volta"]),
    ((1985, "Côte d'Ivoire"), &["Ivory Coast"]),
    // East Asia
    ((1979, "Beijing"), &["Peking"]),
    ((1945, "Taiwan"), &["Formosa"]),
    ((1991, "Ulaanbaatar"), &["Ulan Bator"]),
    // Pacific Island nations
    ((1997, "Samoa"), &["Western Samoa"]),
    ((1980, "Vanuatu"), &["New Hebrides"]),
    // South Asia
    ((1972, "Sri Lanka"), &["Ceylon"]),
    // Ukraine
    ((1992, "Kharkiv"), &["Kharkov"]),
    ((1992, "Kyiv"), &["Kiev"]),
    ((1992, "Luhansk"), &["Lugansk"]),
    ((1992, "Lviv"), &["Lvov"]),
    ((1992, "Odesa"), &["Odessa"]),
    ((1992, "Vinnytsia"), &["Vinnitsa"]),
    ((1992, "Zaporizhzhia"), &["Zaporozhye"]),
];

impl Linter for UpdatePlaceNames {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        let window_size = 3; // 3 tokens ought to be enough for anybody!

        for ch in document.iter_chunks() {
            // TODO let's use a sliding-window iterator over the chunk
            // TODO elsewhere in the codebase i see uses of .tuple_windows - but we want to use a computed window size
            ch.iter().tuple_windows().for_each(|window| {
                
            });

            todo!();
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
    fn update_ayers_rock() {
        assert_suggestion_result(
            "It's dangerous to climb Ayers Rock.",
            UpdatePlaceNames,
            "It's dangerous to climb Uluru.",
        );
    }
}
