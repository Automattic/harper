//! The gender on German noun entries.
//!
//! Gender is the one axis `german_preposition_case.rs` deliberately does not
//! read, because too much of it is wrong. An `-er` ending read as an agent-noun
//! suffix made `Leber`, `Mauer`, `Dauer`, `Nummer`, `Ziffer`, `Metapher` and
//! `Kammer` masculine, and `Fenster`, `Gewitter`, `Kloster` and `Wetter` too.
//! Narrowing a determiner by that turned *"in der Leber"* into an error and
//! took the prose corpus from 15 reports to 310.
//!
//! `scripts/audit_german_gender.py` checks the recorded gender against the
//! articles the corpus puts in front of the word. The entries below are the
//! ones it contradicted with at least five unanimous votes; every one of them
//! was a genuine mistake. They are held here so the classes that produced them
//! cannot come back silently.

#[cfg(test)]
mod tests {
    use crate::language::german::spell::curated_german_dictionary;
    use crate::language::morphology::{Gender, GenderSet, MorphologyExt};
    use crate::spell::Dictionary;

    fn gender(word: &str) -> GenderSet {
        curated_german_dictionary()
            .get_word_metadata_str(word)
            .unwrap_or_else(|| panic!("{word} is not in the German dictionary"))
            .noun_agreement()
            .gender
    }

    /// An `-er` noun is not automatically an agent noun, and an agent noun is
    /// not automatically masculine.
    #[test]
    fn feminine_nouns_in_er_are_feminine() {
        for word in ["Dauer", "Metapher", "Ziffer", "Kammer"] {
            assert_eq!(gender(word), Gender::Feminine.into(), "{word}");
        }
    }

    #[test]
    fn neuter_nouns_are_neuter() {
        for word in [
            "Tier", "Meer", "Papier", "Fenster", "Heer", "Gewitter", "Wetter",
        ] {
            assert!(
                gender(word).contains(GenderSet::NEUTER),
                "{word}: {:?}",
                gender(word)
            );
        }
    }

    #[test]
    fn masculine_nouns_are_not_recorded_neuter_only() {
        for word in ["Raum", "Irrtum"] {
            assert!(
                gender(word).contains(GenderSet::MASCULINE),
                "{word}: {:?}",
                gender(word)
            );
        }
    }

    /// `Bombe` and `Breite` carried masculine *and* neuter at once, which is
    /// two wrong answers rather than an ambiguity.
    #[test]
    fn a_feminine_noun_does_not_carry_two_other_genders() {
        for word in ["Bombe", "Breite"] {
            assert_eq!(gender(word), Gender::Feminine.into(), "{word}");
        }
    }

    /// The corpus can only judge a word it contains often enough. Most of the
    /// dictionary is untested, which is the reason the linter still ignores
    /// this axis — see `readings_allowed_by` in `grammar/determiners.rs`.
    #[test]
    fn the_axis_is_still_mostly_unknown() {
        assert!(gender("Zug").is_empty(), "{:?}", gender("Zug"));
    }
}
