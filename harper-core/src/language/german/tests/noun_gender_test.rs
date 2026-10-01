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

    /// The everyday words the agreement check needs most.
    #[test]
    fn the_commonest_nouns_have_a_gender() {
        for (word, expected) in [
            ("Haus", Gender::Neuter),
            ("Frau", Gender::Feminine),
            ("Tisch", Gender::Masculine),
            ("Mauer", Gender::Feminine),
            ("Theater", Gender::Neuter),
        ] {
            assert_eq!(gender(word), expected.into(), "{word}");
        }
    }

    /// A derivational suffix settles the ones the corpus never reaches. Only
    /// the suffixes that scored at least 90% against the corpus are used;
    /// `-e`, `-er` and `-el`, which scored 65%, 61% and 45%, are not.
    #[test]
    fn derivational_suffixes_settle_the_rest() {
        for word in ["Regierung", "Freiheit", "Möglichkeit", "Wissenschaft"] {
            assert_eq!(gender(word), Gender::Feminine.into(), "{word}");
        }
        for word in ["Mädchen", "Büchlein", "Instrument"] {
            assert!(
                gender(word).contains(GenderSet::NEUTER),
                "{word}: {:?}",
                gender(word)
            );
        }
    }

    /// Most of the evidence is `dem`/`des`/`einem`, which says only "not
    /// feminine". That is worth recording: it rules out a third of the
    /// possibilities, and letting one stray `das` outvote three of them made
    /// *Nutzer* neuter.
    #[test]
    fn weak_evidence_is_recorded_as_weak() {
        assert_eq!(
            gender("Schlüssel"),
            GenderSet::MASCULINE | GenderSet::NEUTER
        );
    }

    /// A compound takes the gender of its last element, and the compound
    /// checker hands it over — including for compounds that have no entry of
    /// their own, which is most of them.
    #[test]
    fn a_compound_inherits_from_its_head() {
        use crate::language::german::spell::combined_german_dictionary;

        let head = |word: &str| {
            combined_german_dictionary()
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("{word} is not recognized"))
                .noun_agreement()
                .gender
        };

        assert_eq!(head("Stadtmauer"), Gender::Feminine.into());
        assert_eq!(
            head("Hausschlüssel"),
            GenderSet::MASCULINE | GenderSet::NEUTER
        );
    }

    /// The corpus can only judge a word it contains often enough, and the
    /// reliable suffixes reach only so far. Half the frequent nouns still have
    /// no gender, which is why the linter does not read this axis yet — see
    /// `readings_allowed_by` in `grammar/determiners.rs`.
    #[test]
    fn the_axis_is_still_incomplete() {
        assert!(gender("Zaun").is_empty(), "{:?}", gender("Zaun"));
    }

    /// The genders that switching the narrowing on exposed.
    ///
    /// Turning gender back on in `readings_allowed_by` took the prose corpus
    /// from 38 reports to 75, and the extra 37 came from barely a dozen words.
    /// Half of them were plain data errors, listed here; the other half were
    /// the head-finder reaching across a clause, which gender only made
    /// visible.
    ///
    /// Every one is an `-er` or an `-e` that was read as an agent noun, which
    /// is the same mistake the file's first test is about.
    #[test]
    fn the_genders_the_narrowing_exposed() {
        for word in [
            "Leber",
            "Aussprache",
            "Angabe",
            "Ansage",
            "Nummer",
            "Schulter",
        ] {
            assert_eq!(
                gender(word),
                GenderSet::FEMININE,
                "{word} is feminine; it was recorded masculine"
            );
        }
        for word in ["Kloster", "Gewässer", "Register"] {
            assert_eq!(
                gender(word),
                GenderSet::NEUTER,
                "{word} is neuter; it was recorded masculine"
            );
        }
    }

    /// Two the corpus itself caught once it had enough text, and two it had
    /// recorded the wrong way round.
    #[test]
    fn the_corpus_caught_these_itself() {
        assert_eq!(gender("Atelier"), GenderSet::NEUTER);
        assert_eq!(gender("Oper"), GenderSet::FEMININE);
        assert_eq!(gender("Smartphone"), GenderSet::NEUTER);
        assert_eq!(gender("Aufschwung"), GenderSet::MASCULINE);
    }

    /// The river, not an agent noun.
    #[test]
    fn a_river_name_is_not_an_agent_noun() {
        assert_eq!(gender("Weser"), GenderSet::FEMININE);
    }
}
