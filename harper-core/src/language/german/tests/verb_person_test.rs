//! Person and number on a finite verb.
//!
//! German conjugation is one affix per ending, and each ending names exactly
//! one person/number combination — or two, which is why the axes are sets. The
//! six flags carried no morphology at all until now, so a verb form said
//! nothing about its subject and subject–verb agreement could not be checked.
//!
//! The pairing is in `annotations.json`: `f` → `-e`, `G` → `-st`, `i` → `-t`,
//! `j` → `-en`, `d` → `-te`, `e` → `-ten`. Two of them are deliberately wider
//! than the indicative alone, and the comments there say why.

#[cfg(test)]
mod tests {
    use crate::language::german::spell::curated_german_dictionary;
    use crate::language::morphology::{MorphologyExt, NumberSet, PersonSet};
    use crate::spell::Dictionary;

    fn verb(word: &str) -> (PersonSet, NumberSet) {
        let agreement = curated_german_dictionary()
            .get_word_metadata_str(word)
            .unwrap_or_else(|| panic!("{word} is not in the German dictionary"))
            .verb_agreement();
        (agreement.person, agreement.number)
    }

    /// `-st` is the one unambiguous present ending German has.
    #[test]
    fn the_second_person_singular_is_exact() {
        for word in ["lernst", "machst", "sagst", "spielst", "fragst"] {
            assert_eq!(
                verb(word),
                (PersonSet::SECOND, NumberSet::SINGULAR),
                "{word} is second person singular and nothing else"
            );
        }
    }

    /// `-t` is third singular and second plural at once.
    #[test]
    fn the_t_ending_holds_two_readings() {
        for word in ["lernt", "macht", "spielt", "fragt"] {
            let (person, number) = verb(word);
            assert!(person.contains(PersonSet::THIRD), "{word}: er {word}");
            assert!(person.contains(PersonSet::SECOND), "{word}: ihr {word}");
            assert!(!person.contains(PersonSet::FIRST), "{word}: not ich");
            assert!(number.contains(NumberSet::SINGULAR));
            assert!(number.contains(NumberSet::PLURAL));
        }
    }

    /// `-e` covers the indicative first person and Konjunktiv I, which is why
    /// the third person is in it. Without that, every line of reported speech
    /// would look like a disagreement.
    #[test]
    fn the_e_ending_allows_reported_speech() {
        for word in ["lerne", "mache", "spiele", "frage"] {
            let (person, number) = verb(word);
            assert!(person.contains(PersonSet::FIRST), "{word}: ich {word}");
            assert!(
                person.contains(PersonSet::THIRD),
                "{word}: er {word} (Konjunktiv I)"
            );
            assert!(!person.contains(PersonSet::SECOND), "{word}: not du");
            assert_eq!(number, NumberSet::SINGULAR);
        }
    }

    /// `-en` is the plural of the present, and the infinitive as well.
    #[test]
    fn the_en_ending_is_plural() {
        for word in ["lernen", "machen", "spielen", "fragen"] {
            let (person, number) = verb(word);
            assert!(person.contains(PersonSet::FIRST), "{word}: wir {word}");
            assert!(person.contains(PersonSet::THIRD), "{word}: sie {word}");
            assert!(!person.contains(PersonSet::SECOND), "{word}: not ihr");
            assert_eq!(number, NumberSet::PLURAL);
        }
    }

    /// The preterite endings split the same way.
    #[test]
    fn the_preterite_endings_split_by_number() {
        for word in ["lernte", "machte", "spielte"] {
            assert_eq!(verb(word).1, NumberSet::SINGULAR, "ich/er {word}");
        }
        for word in ["lernten", "machten", "spielten"] {
            assert_eq!(verb(word).1, NumberSet::PLURAL, "wir/sie {word}");
        }
    }

    /// A subject and a verb that cannot share a person disagree; one that can,
    /// agrees. This is the check the linter is built on.
    #[test]
    fn the_axis_separates_right_from_wrong() {
        let (du_person, du_number) = (PersonSet::SECOND, NumberSet::SINGULAR);

        let (person, number) = verb("lernst");
        assert!(person.intersects(du_person) && number.intersects(du_number));

        let (person, _) = verb("lerne");
        assert!(!person.intersects(du_person), "*du lerne* shares no person");

        let (_, number) = verb("lernen");
        assert!(
            !number.intersects(du_number),
            "*du lernen* shares no number"
        );
    }

    /// Nothing but a verb gained a person: the axis is empty everywhere else,
    /// which is what keeps it from constraining the other rules.
    ///
    /// `die` is in this list, and it took a fix to get there. The verb root
    /// `dien` carried `f`, whose first replacement strips a final `-en`
    /// because that is what an infinitive needs — `lernen` → `lerne`. A root is
    /// not an infinitive, so `dien` became `di` + `e` and the definite article
    /// had carried a verb reading all along; the person axis only made it
    /// visible. `dienen` has an entry of its own and builds `diene` correctly,
    /// so the root does not need the flag.
    #[test]
    fn only_verbs_carry_a_person() {
        let dictionary = curated_german_dictionary();
        for word in ["Mann", "Frau", "Haus", "der", "die", "dieses", "gelb"] {
            let metadata = dictionary
                .get_word_metadata_str(word)
                .unwrap_or_else(|| panic!("{word} is not in the German dictionary"));
            assert!(
                metadata.verb_agreement().person.is_empty(),
                "{word} should carry no person"
            );
        }
    }
}
