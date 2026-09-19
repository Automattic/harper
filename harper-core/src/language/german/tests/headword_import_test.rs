//! igerman98 headwords Harper could not reach at all.
//!
//! Harper's German dictionary is a subset of igerman98's, and the compound
//! checker papers over about half of the difference — the rest were simply
//! reported as misspellings: proper names (`Kalaschnikow`, `Caligula`,
//! `Rijswijk`), place-name derivations, and ordinary vocabulary (`Styropor`,
//! `Parataxe`, `Lokativ`, `Absonderlichkeit`).
//!
//! `scripts/add_german_missing_words.py` imports them, asking `harper-cli`
//! itself which words it cannot reach rather than reimplementing the compound
//! decomposition. Short headwords stay out: any entry of three characters or
//! more becomes a compound element, and a short one is also a plausible typo of
//! a frequent word.

#[cfg(test)]
mod tests {
    use crate::language::german::spell::curated_german_dictionary;
    use crate::spell::Dictionary;

    #[test]
    fn imported_headwords_are_known() {
        let dict = curated_german_dictionary();

        for word in [
            "Kalaschnikow",
            "Caligula",
            "Rijswijk",
            "Styropor",
            "Parataxe",
            "Lokativ",
            "Absonderlichkeit",
            "Allgegenwärtigkeit",
        ] {
            assert!(
                dict.get_word_metadata_str(word).is_some(),
                "'{word}' is an igerman98 headword and should be in the dictionary"
            );
        }
    }

    /// The short ones are deliberately absent — see the module documentation.
    #[test]
    fn short_headwords_stay_out() {
        let dict = curated_german_dictionary();

        for word in ["Aa", "Ahr", "Alf"] {
            assert!(
                dict.get_word_metadata_str(word).is_none(),
                "'{word}' is short enough to be a compound element and a typo; \
                 importing it costs more than it is worth"
            );
        }
    }
}
