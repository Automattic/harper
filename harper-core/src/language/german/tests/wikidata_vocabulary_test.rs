//! The vocabulary `dictionary.dict` takes from the Wikidata lexemes.
//!
//! `scripts/build_german_dictionary.py` writes the word list from the German
//! lexemes of Wikidata (CC0), so ordinary vocabulary and the rarer terms of the
//! sciences are in it without anyone having listed them: *Parataxe*, *Lokativ*,
//! *Styropor*, *Absonderlichkeit*. Proper names come from Wikidata items.
//!
//! What stays out is as deliberate. Any entry of three characters or more
//! becomes a compound element, and a short one is also a plausible typo of a
//! frequent word, so a short noun that few dictionaries list is not taken.

#[cfg(test)]
mod tests {
    use crate::language::german::spell::curated_german_dictionary;
    use crate::spell::Dictionary;

    #[test]
    fn wikidata_vocabulary_is_known() {
        let dict = curated_german_dictionary();

        for word in [
            "Kalaschnikow",
            "Styropor",
            "Parataxe",
            "Hypotaxe",
            "Lokativ",
            "Pulsar",
            "Absonderlichkeit",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is a German lexeme and should be in the dictionary"
            );
        }
    }

    /// The short ones are deliberately absent — see the module documentation.
    #[test]
    fn short_headwords_stay_out() {
        let dict = curated_german_dictionary();

        for word in ["Ahr", "Cer", "Rant"] {
            assert!(
                dict.get_word_metadata_str(word).is_none(),
                "'{word}' is short enough to be a compound element and a typo; \
                 taking it costs more than it is worth"
            );
        }
    }
}
