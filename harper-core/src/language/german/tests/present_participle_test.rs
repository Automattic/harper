//! The declined present participle (`das lernende Kind`) is built by the affix
//! class `c`, which mirrors hunspell's `SFX D`: hang `d`, `de`, `dem`, `den`,
//! `der`, `des` on a verb.
//!
//! Harper's affix classes attach to the *stem*, so `c` takes the infinitive
//! `-en` off first — except after `-ern` and `-eln`, where the `n` belongs to
//! the stem and stays: `entwickeln` gives `entwickelnd`, not `*entwickelend`,
//! and `speichern` gives `speichernd`, not `*speicherend`. Getting that wrong
//! did both kinds of damage at once: it put the non-words in the dictionary and
//! left the real forms out of it, so `entwickelnden` was only ever accepted by
//! the compound decomposition splitting off the article `den`.
//!
//! The flag was also missing from roughly fifteen hundred verbs that do have
//! the form. `scripts/fix_german_verb_forms.py --flags c` restores it, deciding
//! each entry with `hunspell -m`.

#[cfg(test)]
mod tests {
    use crate::language::german::spell::curated_german_dictionary;
    use crate::spell::Dictionary;

    /// Verbs in `-ern` and `-eln` keep their `n` in front of the participle.
    #[test]
    fn participles_of_ern_and_eln_verbs_keep_the_n() {
        let dict = curated_german_dictionary();

        for word in [
            "entwickelnd",
            "entwickelnden",
            "speichernd",
            "speichernden",
            "verändernder",
            "vermittelnden",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is the present participle of an -ern/-eln verb"
            );
        }
    }

    /// And the form the old rule built instead is not a word.
    #[test]
    fn the_stem_stripped_participle_is_not_a_word() {
        let dict = curated_german_dictionary();

        for word in ["entwickelend", "speicherend", "veränderend"] {
            assert!(
                dict.get_word_metadata_str(word).is_none(),
                "'{word}' is not German and must not be in the dictionary"
            );
        }
    }

    /// Ordinary `-en` verbs are unchanged.
    #[test]
    fn participles_of_plain_verbs_still_work() {
        let dict = curated_german_dictionary();

        for word in [
            "lernend",
            "lernende",
            "lernendem",
            "lernenden",
            "lernender",
            "lernendes",
            "denkend",
            "vergeltenden",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is an ordinary present participle"
            );
        }
    }

    /// A participle is an adjective, so it may end a lower-case compound and
    /// must not be read as a noun in running text.
    #[test]
    fn a_participle_is_an_adjective() {
        let dict = curated_german_dictionary();

        for word in ["lernende", "entwickelnde"] {
            let meta = dict.get_word_metadata_str(word).unwrap();
            assert!(
                meta.is_adjective(),
                "'{word}' is a declined participle and should read as an adjective"
            );
        }
    }
}
