mod mass_plurals;
mod noun_countability;

use mass_plurals::MassPlurals;
use noun_countability::NounCountability;

use crate::{
    Document,
    linting::{Lint, Linter},
    remove_overlaps,
    spell::Dictionary,
};

pub struct MassNouns<D> {
    mass_plurals: MassPlurals<D>,
    noun_countability: NounCountability,
}

impl<D> MassNouns<D>
where
    D: Dictionary + Clone,
{
    pub fn new(dict: D) -> Self {
        Self {
            mass_plurals: MassPlurals::new(dict.clone()),
            noun_countability: NounCountability::default(),
        }
    }
}

impl<D> Linter for MassNouns<D>
where
    D: Dictionary,
{
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        lints.extend(self.mass_plurals.lint(document));
        lints.extend(self.noun_countability.lint(document));

        remove_overlaps(&mut lints);

        lints
    }

    fn description(&self) -> &'static str {
        "Detects mass nouns used as countable nouns."
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        linting::tests::{assert_lint_count, assert_suggestion_result},
        spell::FstDictionary,
    };

    use super::MassNouns;

    #[test]
    fn flag_advices_and_an_advice() {
        assert_lint_count(
            "I asked for an advice and he gave me two advices!",
            MassNouns::new(FstDictionary::curated()),
            2,
        );
    }

    #[test]
    fn correct_a_luggage() {
        assert_suggestion_result(
            "I managed to pack all my clothing into one luggage.",
            MassNouns::new(FstDictionary::curated()),
            "I managed to pack all my clothing into one suitcase.",
        );
    }

    #[test]
    fn correct_clothings() {
        assert_suggestion_result(
            "I managed to pack all my clothings into one suitcase.",
            MassNouns::new(FstDictionary::curated()),
            "I managed to pack all my clothing into one suitcase.",
        );
    }
}
